use crate::backend::client::BackendClient;
use crate::config::Config;
use crate::connections::connection::KeyenceConnection;
use crate::connections::read::{send_and_read, ResponseReader};
use crate::error::AdapterError;
use crate::override_role;
use crate::plc::SharedPlc;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use warp::http::StatusCode;
use warp::Filter;

#[derive(Debug, Deserialize)]
pub struct TriggerRequest {
    pub card_id: String,
    pub machine_id: String,
    pub command: String,
    pub decision: String, // "ALLOW" or "DENY"
}

#[derive(Debug, Deserialize)]
pub struct OverrideRequest {
    pub passcode: Option<String>,
    pub command: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TriggerResponse {
    pub status: String,
    pub plc_register: u16,
    pub value: u16,
}

#[derive(Debug, Deserialize)]
pub struct CheckAccessRequest {
    pub card_id: String,
    pub machine_id: String,
    pub command: String,
}

#[derive(Debug, Serialize)]
pub struct CheckAccessResponse {
    pub decision: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthorizeCommandRequest {
    pub card_id: String,
    pub machine_id: String,
    pub command: String,
}

#[derive(Debug, Serialize)]
pub struct AuthorizeCommandResponse {
    pub command: String,
    pub decision: String,
    pub forwarded: bool,
    pub access_checked: bool,
    pub machine_response: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct JwtPayload {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

fn with_auth(adapter_token: String) -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    warp::header::optional("Authorization")
        .and(warp::any().map(move || adapter_token.clone()))
        .and_then(|auth_header: Option<String>, secret: String| async move {
            let auth = auth_header.ok_or_else(|| warp::reject::custom(AdapterError::Auth))?;
            let token = auth
                .strip_prefix("Bearer ")
                .ok_or_else(|| warp::reject::custom(AdapterError::Auth))?;

            if token == secret {
                return Ok(());
            }

            let decoding_key = DecodingKey::from_secret(secret.as_bytes());
            let validation = Validation::new(Algorithm::HS256);

            decode::<JwtPayload>(token, &decoding_key, &validation)
                .map(|_| ())
                .map_err(|_| warp::reject::custom(AdapterError::Auth))
        })
        .untuple_one()
}

fn apply_plc_signal(
    plc: SharedPlc,
    config: &Config,
    decision: &str,
) -> Result<TriggerResponse, AdapterError> {
    let mut plc_guard = plc.lock().map_err(|_| AdapterError::Plc)?;

    let (register, value) = match decision {
        "ALLOW" => {
            plc_guard.set_allow()?;
            (config.plc_register_allow, 1)
        }
        "DENY" => {
            plc_guard.set_deny()?;
            (config.plc_register_deny, 1)
        }
        _ => return Err(AdapterError::Plc),
    };

    Ok(TriggerResponse {
        status: "OK".to_string(),
        plc_register: register,
        value,
    })
}

fn is_access_controlled_command(command: &str) -> bool {
    matches!(command, "R0" | "RO" | "S0" | "SO")
}

fn normalize_command(command: &str) -> String {
    match command.trim().to_ascii_uppercase().as_str() {
        "RO" | "R0" => "R0".to_string(),
        "SO" | "S0" => "S0".to_string(),
        other => other.to_string(),
    }
}

fn forward_keyence_command(cfg: &Config, command: &str) -> Result<Option<String>, AdapterError> {
    let mut conn = KeyenceConnection::new(&cfg.keyence_host, cfg.keyence_port)?;
    let response = send_and_read(&mut conn, command)?;

    if response.is_empty() {
        return Ok(None);
    }

    if let Some(error) = ResponseReader::parse_error(&response) {
        return Err(AdapterError::PlcComm(format!(
            "machine rejected command {}: {}",
            command, error
        )));
    }

    if !ResponseReader::is_success(&response) {
        return Err(AdapterError::PlcComm(format!(
            "unexpected machine response for {}: {}",
            command, response
        )));
    }

    Ok(Some(response))
}

async fn check_and_apply_access(
    request: CheckAccessRequest,
    cfg: Config,
    plc: SharedPlc,
) -> Result<CheckAccessResponse, AdapterError> {
    let pending_started_at = Instant::now();

    {
        let mut plc_guard = plc.lock().map_err(|_| AdapterError::Plc)?;
        plc_guard.set_request_pending()?;
    }

    let backend = BackendClient::new(&cfg);
    let allowed = backend
        .check_access(&request.card_id, &request.machine_id, &request.command)
        .await?;

    let min_pending = Duration::from_millis(cfg.plc_request_pending_min_ms);
    if pending_started_at.elapsed() < min_pending {
        sleep(min_pending - pending_started_at.elapsed()).await;
    }

    {
        let mut plc_guard = plc.lock().map_err(|_| AdapterError::Plc)?;
        if allowed {
            plc_guard.set_allow()?;
        } else {
            plc_guard.set_deny()?;
        }
        plc_guard.clear_request_pending()?;
    }

    Ok(CheckAccessResponse {
        decision: if allowed { "ALLOW" } else { "DENY" }.to_string(),
    })
}

async fn authorize_and_forward_command(
    request: AuthorizeCommandRequest,
    cfg: Config,
    plc: SharedPlc,
) -> Result<AuthorizeCommandResponse, AdapterError> {
    let command = normalize_command(&request.command);

    if !is_access_controlled_command(&command) {
        let machine_response = forward_keyence_command(&cfg, &command)?;
        return Ok(AuthorizeCommandResponse {
            command,
            decision: "BYPASS".to_string(),
            forwarded: true,
            access_checked: false,
            machine_response,
        });
    }

    let pending_started_at = Instant::now();
    {
        let mut plc_guard = plc.lock().map_err(|_| AdapterError::Plc)?;
        plc_guard.set_request_pending()?;
    }

    let backend = BackendClient::new(&cfg);
    let allowed_result = backend
        .check_access(&request.card_id, &request.machine_id, &command)
        .await;

    let min_pending = Duration::from_millis(cfg.plc_request_pending_min_ms);
    if pending_started_at.elapsed() < min_pending {
        sleep(min_pending - pending_started_at.elapsed()).await;
    }

    let allowed = match allowed_result {
        Ok(value) => value,
        Err(err) => {
            let mut plc_guard = plc.lock().map_err(|_| AdapterError::Plc)?;
            plc_guard.clear_request_pending()?;
            return Err(err);
        }
    };

    let forward_result = if allowed {
        forward_keyence_command(&cfg, &command)
    } else {
        Ok(None)
    };

    let mut plc_guard = plc.lock().map_err(|_| AdapterError::Plc)?;
    if allowed && forward_result.is_ok() {
        plc_guard.set_allow()?;
    } else {
        plc_guard.set_deny()?;
    }
    plc_guard.clear_request_pending()?;
    let machine_response = forward_result?;
    let forwarded = allowed;

    Ok(AuthorizeCommandResponse {
        command,
        decision: if allowed { "ALLOW" } else { "DENY" }.to_string(),
        forwarded,
        access_checked: true,
        machine_response,
    })
}

pub fn create_filters(
    config: Config,
    plc: SharedPlc,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let health = warp::path!("health").map(|| "OK");

    let api = warp::path("api");
    let auth = with_auth(config.adapter_token.clone());

    let trigger_plc_plc = plc.clone();
    let trigger_plc_cfg = config.clone();
    let trigger_plc = warp::path!("trigger")
        .and(warp::post())
        .and(auth.clone())
        .and(warp::body::json())
        .and(warp::any().map(move || trigger_plc_plc.clone()))
        .and(warp::any().map(move || trigger_plc_cfg.clone()))
        .and_then(
            |request: TriggerRequest, plc: SharedPlc, cfg: Config| async move {
                let _ = (&request.card_id, &request.machine_id, &request.command);

                apply_plc_signal(plc, &cfg, request.decision.as_str())
                    .map(|payload| {
                        warp::reply::with_status(warp::reply::json(&payload), StatusCode::OK)
                    })
                    .map_err(warp::reject::custom)
            },
        );

    let override_cfg = config.clone();
    let override_plc = plc.clone();
    let trigger_override = warp::path!("override")
        .and(warp::post())
        .and(warp::header::optional::<String>("X-Override-Token"))
        .and(warp::body::json())
        .and_then(
            move |override_token: Option<String>, request: OverrideRequest| {
                let cfg = override_cfg.clone();
                let plc = override_plc.clone();

                async move {
                    let passcode = request.passcode.as_deref();
                    let token = override_token.as_deref();

                    if !override_role::is_override_authorized(&cfg, token, passcode) {
                        return Err(warp::reject::custom(AdapterError::OverrideAuth));
                    }

                    let decision = request.command.as_deref().unwrap_or("ALLOW");
                    let _ = request.reason.as_deref();

                    apply_plc_signal(plc, &cfg, decision)
                        .map(|payload| {
                            warp::reply::with_status(warp::reply::json(&payload), StatusCode::OK)
                        })
                        .map_err(warp::reject::custom)
                }
            },
        );

    let check_access_plc = plc.clone();
    let check_access_cfg = config.clone();
    let check_access = warp::path!("check-access")
        .and(warp::post())
        .and(warp::query::<CheckAccessRequest>())
        .and(warp::any().map(move || check_access_plc.clone()))
        .and(warp::any().map(move || check_access_cfg.clone()))
        .and_then(
            |request: CheckAccessRequest, plc: SharedPlc, cfg: Config| async move {
                check_and_apply_access(request, cfg, plc)
                    .await
                    .map(|payload| {
                        warp::reply::with_status(warp::reply::json(&payload), StatusCode::OK)
                    })
                    .map_err(warp::reject::custom)
            },
        );

    let authorize_cmd_plc = plc.clone();
    let authorize_cmd_cfg = config.clone();
    let authorize_command = warp::path!("authorize-command")
        .and(warp::post())
        .and(auth.clone())
        .and(warp::body::json())
        .and(warp::any().map(move || authorize_cmd_plc.clone()))
        .and(warp::any().map(move || authorize_cmd_cfg.clone()))
        .and_then(
            |request: AuthorizeCommandRequest, plc: SharedPlc, cfg: Config| async move {
                authorize_and_forward_command(request, cfg, plc)
                    .await
                    .map(|payload| {
                        warp::reply::with_status(warp::reply::json(&payload), StatusCode::OK)
                    })
                    .map_err(warp::reject::custom)
            },
        );

    let routes = api.and(
        health
            .or(trigger_plc)
            .or(trigger_override)
            .or(check_access)
            .or(authorize_command),
    );

    routes.recover(handle_rejection)
}

async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(AdapterError::Auth) = err.find() {
        let body = ErrorResponse {
            error: "authentication failed".to_string(),
        };
        return Ok(warp::reply::with_status(
            warp::reply::json(&body),
            StatusCode::UNAUTHORIZED,
        ));
    }

    if let Some(AdapterError::OverrideAuth) = err.find() {
        let body = ErrorResponse {
            error: "override token or passcode is invalid".to_string(),
        };
        return Ok(warp::reply::with_status(
            warp::reply::json(&body),
            StatusCode::FORBIDDEN,
        ));
    }

    if let Some(err) = err.find::<AdapterError>() {
        let body = ErrorResponse {
            error: err.to_string(),
        };
        return Ok(warp::reply::with_status(
            warp::reply::json(&body),
            StatusCode::BAD_REQUEST,
        ));
    }

    Err(err)
}

pub async fn start_server(config: Config, plc: SharedPlc) -> Result<(), AdapterError> {
    let routes = create_filters(config.clone(), plc);
    let addr = format!("{}:{}", config.server_host, config.server_port);

    println!("Starting HTTP server on {}", addr);
    let socket_addr: std::net::SocketAddr = addr.parse().map_err(|_| AdapterError::Config)?;
    if std::net::TcpListener::bind(socket_addr).is_ok() {
        warp::serve(routes).run(socket_addr).await;
        return Ok(());
    }

    let fallback_addr = std::net::SocketAddr::from(([0, 0, 0, 0], config.server_port));
    eprintln!(
        "Server host {} is not available, falling back to {}",
        config.server_host, fallback_addr
    );
    warp::serve(routes).run(fallback_addr).await;

    Ok(())
}
