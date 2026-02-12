use thiserror::Error;
use warp::reject::Reject;

#[derive(Error, Debug)]
pub enum AdapterError {
    #[error("Backend error")]
    Backend,

    #[error("PLC error")]
    Plc,

    #[error("Network timeout")]
    Timeout,

    #[error("Serial communication error: {0}")]
    Serial(String),

    #[error("PLC communication error: {0}")]
    PlcComm(String),

    #[error("Invalid configuration")]
    Config,

    #[error("Authentication failed")]
    Auth,
}

impl Reject for AdapterError {}
