use crate::config::Config;
use crate::error::AdapterError;
use crate::plc::PlcDevice;
use std::io::Write;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

pub struct CognexPlc {
    config: Config,
}

impl CognexPlc {
    pub fn new(cfg: &Config) -> Self {
        Self {
            config: cfg.clone(),
        }
    }

    fn send_command(&self, command: &str) -> Result<(), AdapterError> {
        let address = format!("{}:{}", self.config.cognex_host, self.config.cognex_port);
        let socket = address
            .to_socket_addrs()
            .map_err(|e| AdapterError::PlcComm(e.to_string()))?
            .next()
            .ok_or(AdapterError::Plc)?;

        let mut stream = TcpStream::connect_timeout(&socket, Duration::from_secs(2))
            .map_err(|e| AdapterError::PlcComm(e.to_string()))?;
        stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .map_err(|e| AdapterError::PlcComm(e.to_string()))?;

        let payload = format!("{}\n", command);
        stream
            .write_all(payload.as_bytes())
            .map_err(|e| AdapterError::PlcComm(e.to_string()))
    }
}

impl PlcDevice for CognexPlc {
    fn set_allow(&mut self) -> Result<(), AdapterError> {
        println!(
            "Cognex: ACCESS ALLOWED - sending '{}' to {}:{}",
            self.config.cognex_allow_command, self.config.cognex_host, self.config.cognex_port
        );
        self.send_command(&self.config.cognex_allow_command)
    }

    fn set_deny(&mut self) -> Result<(), AdapterError> {
        println!(
            "Cognex: ACCESS DENIED - sending '{}' to {}:{}",
            self.config.cognex_deny_command, self.config.cognex_host, self.config.cognex_port
        );
        self.send_command(&self.config.cognex_deny_command)
    }

    fn reset_signals(&mut self) -> Result<(), AdapterError> {
        println!(
            "Cognex: RESET SIGNALS - sending '{}' to {}:{}",
            self.config.cognex_reset_command, self.config.cognex_host, self.config.cognex_port
        );
        self.send_command(&self.config.cognex_reset_command)
    }
}
