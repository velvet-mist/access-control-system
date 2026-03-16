use crate::backend::client::BackendClient;
use crate::config::Config;
use crate::connections::shared::SharedKeyence;
use crate::error::AdapterError;
use crate::override_role;
use crate::plc::SharedPlc;
use crate::tcp_handler::tcp::{grant_access, AccessState};
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
pub struct MachineSequenceResponse {
    pub command: String,
    pub response: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthorizeCommandResponse {
    pub command: String,
    pub decision: String,
    pub forwarded: bool,
    pub access_checked: bool,
    pub machine_response: Option<String>,
    pub machine_sequence: Vec<MachineSequenceResponse>,
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
        "INSPECT" | "TRIGGER_INSPECTION" => "INSPECT".to_string(),
        other => other.to_string(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CommandPermission {
    Checked,
    Unchecked,
}

/// Send a single command to the Keyence unit over the shared persistent connection.
async fn forward_keyence_command(
    keyence: &SharedKeyence,
    command: &str,
    permission: CommandPermission,
) -> Result<Option<String>, AdapterError> {
    if permission == CommandPermission::Unchecked && is_access_controlled_command(command) {
        return Err(AdapterError::ProtectedCommand(command.to_string()));
    }

    let response = keyence.send(command).await?;

    if response.is_empty() {
        return Ok(None);
    }

    Ok(Some(response))
}

/// Send a sequence of commands over the same shared connection.
async fn run_machine_sequence(
    keyence: &SharedKeyence,
    commands: &[&str],
    permission: CommandPermission,
) -> Result<Vec<MachineSequenceResponse>, AdapterError> {
    let mut responses = Vec::with_capacity(commands.len());

    for command in commands {
        if permission == CommandPermission::Unchecked && is_access_controlled_command(command) {
            return Err(AdapterError::ProtectedCommand((*command).to_string()));
        }

        let response = keyence.send(command).await?;

        responses.push(MachineSequenceResponse {
            command: (*command).to_string(),
            response: if response.is_empty() {
                None
            } else {
                Some(response)
            },
        });
    }

    Ok(responses)
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
    keyence: SharedKeyence,
    access: AccessState,
) -> Result<AuthorizeCommandResponse, AdapterError> {
    let command = normalize_command(&request.command);

    // Non-access-controlled commands (e.g. INSPECT) bypass the backend check
    if !is_access_controlled_command(&command) {
        let machine_sequence = if command == "INSPECT" {
            run_machine_sequence(&keyence, &["RS", "TA"], CommandPermission::Unchecked).await?
        } else {
            let machine_response =
                forward_keyence_command(&keyence, &command, CommandPermission::Unchecked).await?;
            vec![MachineSequenceResponse {
                command: command.clone(),
                response: machine_response.clone(),
            }]
        };

        return Ok(AuthorizeCommandResponse {
            command,
            decision: "BYPASS".to_string(),
            forwarded: true,
            access_checked: false,
            machine_response: machine_sequence.last().and_then(|step| step.response.clone()),
            machine_sequence,
        });
    }

    // R0 / S0 — check backend, then grant or deny
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

    let machine_sequence = if allowed {
        // Grant access so the TCP listener will forward the next R0/S0 from Keyence
        grant_access(&access);
        println!("Access granted for {} — TCP listener will allow next R0/S0", command);

        let response =
            forward_keyence_command(&keyence, &command, CommandPermission::Checked).await?;
        response
            .map(|value| {
                vec![MachineSequenceResponse {
                    command: command.clone(),
                    response: Some(value),
                }]
            })
            .unwrap_or_default()
    } else {
        vec![]
    };

    let mut plc_guard = plc.lock().map_err(|_| AdapterError::Plc)?;
    if allowed {
        plc_guard.set_allow()?;
    } else {
        plc_guard.set_deny()?;
    }
    plc_guard.clear_request_pending()?;
    let forwarded = allowed;

    Ok(AuthorizeCommandResponse {
        command,
        decision: if allowed { "ALLOW" } else { "DENY" }.to_string(),
        forwarded,
        access_checked: true,
        machine_response: machine_sequence.first().and_then(|step| step.response.clone()),
        machine_sequence,
    })
}

pub fn create_filters(
    config: Config,
    plc: SharedPlc,
    keyence: SharedKeyence,
    access: AccessState,
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
    let authorize_cmd_keyence = keyence.clone();
    let authorize_cmd_access = access.clone();
    let authorize_command_handler = warp::post()
        .and(auth.clone())
        .and(warp::body::json())
        .and(warp::any().map(move || authorize_cmd_plc.clone()))
        .and(warp::any().map(move || authorize_cmd_cfg.clone()))
        .and(warp::any().map(move || authorize_cmd_keyence.clone()))
        .and(warp::any().map(move || authorize_cmd_access.clone()))
        .and_then(
            |request: AuthorizeCommandRequest,
             plc: SharedPlc,
             cfg: Config,
             keyence: SharedKeyence,
             access: AccessState| async move {
                authorize_and_forward_command(request, cfg, plc, keyence, access)
                    .await
                    .map(|payload| {
                        warp::reply::with_status(warp::reply::json(&payload), StatusCode::OK)
                    })
                    .map_err(warp::reject::custom)
            },
        );

    let authorize_command = warp::path!("authorize-command").and(authorize_command_handler.clone());
    let machine_command = warp::path!("machine-command").and(authorize_command_handler);

    let routes = api.and(
        health
            .or(trigger_plc)
            .or(trigger_override)
            .or(check_access)
            .or(machine_command)
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

pub async fn start_server(
    config: Config,
    plc: SharedPlc,
    access: AccessState,
) -> Result<(), AdapterError> {
    // One shared persistent Keyence TCP connection for the lifetime of the server
    let keyence = SharedKeyence::new(&config.keyence_host, config.keyence_port);

    let routes = create_filters(config.clone(), plc, keyence, access);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protected_commands_are_normalized() {
        assert_eq!(normalize_command("ro"), "R0");
        assert_eq!(normalize_command("SO"), "S0");
    }

    #[test]
    fn inspect_is_normalized() {
        assert_eq!(normalize_command("TRIGGER_INSPECTION"), "INSPECT");
        assert_eq!(normalize_command("inspect"), "INSPECT");
    }

    #[test]
    fn access_controlled_detection() {
        assert!(is_access_controlled_command("R0"));
        assert!(is_access_controlled_command("S0"));
        assert!(is_access_controlled_command("RO")); // OCR alias
        assert!(!is_access_controlled_command("INSPECT"));
        assert!(!is_access_controlled_command("TA"));
    }
}