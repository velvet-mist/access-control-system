pub mod dummy_tcp_plc;
pub mod keyence;
pub mod keyence_tcp;

use crate::config::Config;
use crate::error::AdapterError;
use dummy_tcp_plc::DummyTcpPlc;
use std::sync::{Arc, Mutex};

pub trait PlcDevice: Send + Sync {
    fn set_request_pending(&mut self) -> Result<(), AdapterError> { Ok(()) }
    fn clear_request_pending(&mut self) -> Result<(), AdapterError> { Ok(()) }
    fn set_allow(&mut self) -> Result<(), AdapterError>;
    fn set_deny(&mut self) -> Result<(), AdapterError>;
    #[allow(dead_code)]
    fn reset_signals(&mut self) -> Result<(), AdapterError>;
}

pub type SharedPlc = Arc<Mutex<Box<dyn PlcDevice>>>;

pub fn create_plc_device(cfg: &Config) -> Result<SharedPlc, AdapterError> {
    if cfg.uses_plc_tcp() {
        // TCP transport → dummy (proxy spawned separately)
        Ok(Arc::new(Mutex::new(Box::new(DummyTcpPlc))))
    } else {
        // Serial transport → Modbus RTU
        Ok(Arc::new(Mutex::new(Box::new(
            keyence::KeyencePlc::new(cfg),
        ))))
    }
}
