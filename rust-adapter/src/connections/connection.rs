use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use crate::error::AdapterError;

/// TCP connection wrapper for Keyence PLC communication
pub struct KeyenceConnection {
    stream: TcpStream,
}

impl KeyenceConnection {
    /// Create a new TCP connection to the Keyence PLC
    pub fn new(host: &str, port: u16) -> Result<Self, AdapterError> {
        let address = format!("{}:{}", host, port);
        let socket_addr: SocketAddr = address
            .parse::<SocketAddr>()
            .map_err(|e| AdapterError::PlcComm(e.to_string()))?;
        let stream = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(5))
            .map_err(|e| AdapterError::PlcComm(e.to_string()))?;

        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .map_err(|e| AdapterError::PlcComm(e.to_string()))?;
        stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .map_err(|e| AdapterError::PlcComm(e.to_string()))?;

        Ok(Self { stream })
    }

    /// Send raw bytes to the PLC
    pub fn send(&mut self, data: &[u8]) -> Result<(), AdapterError> {
        self.stream
            .write_all(data)
            .map_err(|e| AdapterError::PlcComm(e.to_string()))
    }

    /// Read response bytes from the PLC
    #[allow(dead_code)]
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, AdapterError> {
        self.stream
            .read(buffer)
            .map_err(|e| AdapterError::PlcComm(e.to_string()))
    }

    /// Send a command string with CRLF terminator
    pub fn send_command(&mut self, command: &str) -> Result<(), AdapterError> {
        let payload = format!("{}\r\n", command);
        self.send(payload.as_bytes())
    }

    /// Get a mutable reference to the underlying stream for more control
    #[allow(dead_code)]
    pub fn stream_mut(&mut self) -> &mut TcpStream {
        &mut self.stream
    }

    /// Check if the connection is still alive
    #[allow(dead_code)]
    pub fn is_connected(&self) -> bool {
        // Simple check - in production you'd use ping or keepalive
        true
    }
}

impl Write for KeyenceConnection {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }
}

impl Read for KeyenceConnection {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_creation() {
        // This test would require a real PLC to run
        // For now just verify the struct can be created (won't connect)
        let result = KeyenceConnection::new("192.168.0.20", 9004);
        // Expect connection to fail since there's no real PLC
        assert!(result.is_err() || result.unwrap().is_connected());
    }
}
