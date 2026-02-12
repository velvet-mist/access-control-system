use http_client::http_types::auth;
use warp::Filter;
use serde::{Deserialize, Serialize};
use crate::error::AdapterError;
use crate::config::Config;
use crate::plc::keyence::KeyencePlc;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};


#[derive(Debug, Deserialize)]
pub struct TriggerRequest {
    pub card_id: String,
    pub machine_id: String,
    pub command: String,
    pub decision: String, // "ALLOW" or "DENY"
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
pub struct JwtPayload {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

pub fn create_filters(
    config: Config,
    mut plc: KeyencePlc,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let health = warp::path!("health").map(|| "OK");

    let api = warp::path("api");

    // JWT authentication middleware
    let jwt_secret = config.adapter_token.clone();
    let auth = warp::header::optional("Authorization")
        .and(warp::any().map(move || jwt_secret.clone()))
        .and_then(|auth_header: Option<String>, secret: String| async move {
            if auth_header.is_none() {
                return Err(warp::reject::custom(AdapterError::Auth));
            }
            
            let token = auth_header.unwrap();
            let token = token.strip_prefix("Bearer ").ok_or(AdapterError::Auth)?;
            
            let decoding_key = DecodingKey::from_secret(secret.as_bytes());
            let validation = Validation::new(Algorithm::HS256);
            
            match decode::<JwtPayload>(token, &decoding_key, &validation) {
                Ok(_) => Ok(()),
                Err(_) => Err(warp::reject::custom(AdapterError::Auth)),
            }
        });

    // PLC trigger endpoint
    let triggers = warp::path!("trigger")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || plc.clone()))
        .and_then(
            |request: TriggerRequest, plc: KeyencePlc| async move {
                let (register, value) = match request.decision.as_str() {
                    "ALLOW" => (100, 1),  // Configurable
                    "DENY" => (101, 1),   // Configurable
                    _ => return Err(warp::reject::custom(AdapterError::Plc)),
                };
                
                let mut plc = plc;
                let result = if request.decision == "ALLOW" {
                    plc.set_allow()
                } else {
                    plc.set_deny()
                };
                
                match result {
                    Ok(_) => Ok(warp::reply::json(&TriggerResponse {
                        status: "OK".to_string(),
                        plc_register: register,
                        value,
                    })),
                    Err(e) => Err(warp::reject::custom(e)),
                }
            },
        );

    // Check access endpoint (called by Python backend)
    let check_access = warp::path!("check-access")
        .and(warp::query::<CheckAccessRequest>())
        .and_then(|request: CheckAccessRequest| async move {
            // Just echo the decision for now - real implementation would check with backend
            Ok::<_, warp::Rejection>(warp::reply::json(&CheckAccessResponse {
                decision: "ALLOW".to_string(), // Placeholder
            }))
        });

    // Combined routes
    api.and(health.or(triggers).or(check_access))
}

pub async fn start_server(config: Config, plc: KeyencePlc) -> Result<(), AdapterError> {
    let routes = create_filters(config.clone(), plc);
    let addr = format!("{}:{}", config.server_host, config.server_port);
    
    println!("Starting HTTP server on {}", addr);
    let socket_addr: std::net::SocketAddr = addr.parse().map_err(|_| AdapterError::Config)?;
    warp::serve(routes)
        .run(socket_addr)
        .await;
    
    Ok(())
}

