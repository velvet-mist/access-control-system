use crate::error::AdapterError;
use crate::plc::PlcDevice;

pub struct DummyTcpPlc;

impl PlcDevice for DummyTcpPlc {
    fn set_allow(&mut self) -> Result<(), AdapterError> {
        println!("TCP proxy: ALLOW signal (handled by access state/proxy)");
        Ok(())
    }

    fn set_deny(&mut self) -> Result<(), AdapterError> {
        println!("TCP proxy: DENY signal (handled by access state/proxy)");
        Ok(())
    }

    fn reset_signals(&mut self) -> Result<(), AdapterError> {
        Ok(())
    }
}
