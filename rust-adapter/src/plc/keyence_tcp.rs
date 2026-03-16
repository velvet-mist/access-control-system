use crate::config::Config;
use crate::error::AdapterError;
use crate::plc::PlcDevice;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

pub struct KeyenceTcpPlc {
    config: Config,
}

impl KeyenceTcpPlc {
    pub fn new(cfg: &Config) -> Self {
        Self {
            config: cfg.clone(),
        }
    }

    fn connect(&self) -> Result<TcpStream, AdapterError> {
        let addr = format!("{}:{}", self.config.keyence_host, self.config.keyence_port);
        TcpStream::connect(&addr)
            .map_err(|e| AdapterError::PlcComm(e.to_string()))
            .and_then(|s| {
                s.set_read_timeout(Some(Duration::from_secs(2)))
                    .map_err(|e| AdapterError::PlcComm(e.to_string()))?;
                s.set_write_timeout(Some(Duration::from_secs(2)))
                    .map_err(|e| AdapterError::PlcComm(e.to_string()))?;
                Ok(s)
            })
    }

    fn write_register(&self, register: u16, value: u16) -> Result<(), AdapterError> {
        // Modbus TCP frame
        let mut frame = Vec::with_capacity(12);
        // MBAP header
        frame.extend_from_slice(&[0x00, 0x01]); // transaction id
        frame.extend_from_slice(&[0x00, 0x00]); // protocol id
        frame.extend_from_slice(&[0x00, 0x06]); // length (6 bytes follow)
        frame.push(self.config.plc_slave_addr);  // unit id
        // PDU
        frame.push(0x06);                        // function: write single register
        frame.push((register >> 8) as u8);
        frame.push((register & 0xFF) as u8);
        frame.push((value >> 8) as u8);
        frame.push((value & 0xFF) as u8);

        let mut stream = self.connect()?;
        stream.write_all(&frame)
            .map_err(|e| AdapterError::PlcComm(e.to_string()))?;

        // Read response (should be 12 bytes echo)
        let mut buf = [0u8; 12];
        stream.read(&mut buf)
            .map_err(|e| AdapterError::PlcComm(e.to_string()))?;

        Ok(())
    }
}

impl PlcDevice for KeyenceTcpPlc {
    fn set_request_pending(&mut self) -> Result<(), AdapterError> {
        println!(
            "PLC: ACCESS REQUEST PENDING - Writing to register {}",
            self.config.plc_register_request_pending
        );
        self.write_register(self.config.plc_register_request_pending, 1)
    }

    fn clear_request_pending(&mut self) -> Result<(), AdapterError> {
        self.write_register(self.config.plc_register_request_pending, 0)
    }

    fn set_allow(&mut self) -> Result<(), AdapterError> {
        println!(
            "PLC: ACCESS ALLOWED - Writing to register {}",
            self.config.plc_register_allow
        );
        self.write_register(self.config.plc_register_allow, 1)
    }

    fn set_deny(&mut self) -> Result<(), AdapterError> {
        println!(
            "PLC: ACCESS DENIED - Writing to register {}",
            self.config.plc_register_deny
        );
        self.write_register(self.config.plc_register_deny, 1)
    }

    fn reset_signals(&mut self) -> Result<(), AdapterError> {
        self.clear_request_pending()?;
        self.write_register(self.config.plc_register_allow, 0)?;
        self.write_register(self.config.plc_register_deny, 0)
    }
}