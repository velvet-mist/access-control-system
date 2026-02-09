use thiserror::Error;

#[derive(Error, Debug)]
pub enum AdapterError {
    #[error("Backend error")]
    Backend,

    #[error("PLC error")]
    Plc,

    #[error("Network timeout")]
    Timeout,
}
