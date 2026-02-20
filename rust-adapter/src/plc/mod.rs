pub mod cognex;
pub mod keyence;

use crate::config::Config;
use crate::error::AdapterError;
use std::sync::{Arc, Mutex};

pub trait PlcDevice: Send + Sync {
    fn set_allow(&mut self) -> Result<(), AdapterError>;
    fn set_deny(&mut self) -> Result<(), AdapterError>;
    fn reset_signals(&mut self) -> Result<(), AdapterError>;
}

pub type SharedPlc = Arc<Mutex<Box<dyn PlcDevice>>>;

pub fn create_plc_device(cfg: &Config) -> Result<SharedPlc, AdapterError> {
    match cfg.plc_type.as_str() {
        "keyence" => Ok(Arc::new(Mutex::new(Box::new(keyence::KeyencePlc::new(cfg))))),
        "cognex" => Ok(Arc::new(Mutex::new(Box::new(cognex::CognexPlc::new(cfg))))),
        _ => Err(AdapterError::Config),
    }
}
