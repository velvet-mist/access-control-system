#![allow(dead_code)]

use crate::connections::connection::KeyenceConnection;
use crate::error::AdapterError;
use std::io::Read;

/// Response reader for parsing Keyence PLC responses
pub struct ResponseReader {
    buffer: Vec<u8>,
    timeout_ms: u64,
}

impl ResponseReader {
    /// Create a new response reader
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            timeout_ms: 2000,
        }
    }

    /// Create a response reader with custom timeout
    pub fn with_timeout(timeout_ms: u64) -> Self {
        Self {
            buffer: Vec::new(),
            timeout_ms,
        }
    }

    /// Read a response from the connection
    pub fn read_response(&mut self, conn: &mut KeyenceConnection) -> Result<String, AdapterError> {
        self.buffer.clear();

        let mut temp_buffer = [0u8; 256];
        let mut found_crlf = false;

        // Read until we find CRLF or timeout
        while !found_crlf {
            match conn.stream_mut().read(&mut temp_buffer) {
                Ok(0) => {
                    // Connection closed
                    break;
                }
                Ok(n) => {
                    self.buffer.extend_from_slice(&temp_buffer[..n]);

                    // Check for CRLF
                    if self.buffer.len() >= 2 {
                        let len = self.buffer.len();
                        if self.buffer[len - 2] == b'\r' && self.buffer[len - 1] == b'\n' {
                            found_crlf = true;
                        }
                    }

                    // Safety limit
                    if self.buffer.len() > 1024 {
                        return Err(AdapterError::PlcComm(
                            "Response buffer overflow".to_string(),
                        ));
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut
                    {
                        // Timeout - return what we have
                        break;
                    }
                    return Err(AdapterError::PlcComm(e.to_string()));
                }
            }
        }

        // Remove trailing CRLF if present
        let response = String::from_utf8_lossy(&self.buffer);
        let response = response
            .trim_end_matches("\r\n")
            .trim_end_matches("\n")
            .trim_end_matches("\r");

        Ok(response.to_string())
    }

    /// Read a specific number of bytes
    pub fn read_bytes(
        &mut self,
        conn: &mut KeyenceConnection,
        count: usize,
    ) -> Result<Vec<u8>, AdapterError> {
        self.buffer.clear();
        self.buffer.resize(count, 0);

        let mut total_read = 0;
        while total_read < count {
            match conn.stream_mut().read(&mut self.buffer[total_read..]) {
                Ok(0) => {
                    // Connection closed
                    break;
                }
                Ok(n) => {
                    total_read += n;
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut
                    {
                        break;
                    }
                    return Err(AdapterError::PlcComm(e.to_string()));
                }
            }
        }

        Ok(self.buffer.clone())
    }

    /// Read response and parse as lines
    pub fn read_lines(
        &mut self,
        conn: &mut KeyenceConnection,
    ) -> Result<Vec<String>, AdapterError> {
        self.buffer.clear();

        let mut temp_buffer = [0u8; 128];

        loop {
            match conn.stream_mut().read(&mut temp_buffer) {
                Ok(0) => break,
                Ok(n) => {
                    self.buffer.extend_from_slice(&temp_buffer[..n]);
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut
                    {
                        break;
                    }
                    return Err(AdapterError::PlcComm(e.to_string()));
                }
            }
        }

        let response = String::from_utf8_lossy(&self.buffer);
        let lines: Vec<String> = response
            .lines()
            .map(|s| s.trim_end_matches("\r").to_string())
            .collect();

        Ok(lines)
    }

    /// Check if response indicates success (typically "OK" or starts with specific code)
    pub fn is_success(response: &str) -> bool {
        let upper = response.to_uppercase();
        upper == "OK"
            || upper.starts_with("OK")
            || upper.starts_with("0")
            || upper.starts_with("20")
    }

    /// Parse error code from response
    pub fn parse_error(response: &str) -> Option<String> {
        // Keyence typically returns error codes like "ER1", "ER2", etc.
        if response.starts_with("ER") {
            Some(response.to_string())
        } else if response.to_uppercase().starts_with("ERROR") {
            Some(response.to_string())
        } else {
            None
        }
    }
}

impl Default for ResponseReader {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to send command and read response
pub fn send_and_read(conn: &mut KeyenceConnection, command: &str) -> Result<String, AdapterError> {
    conn.send_command(command)?;

    let mut reader = ResponseReader::new();
    reader.read_response(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_reader_creation() {
        let reader = ResponseReader::new();
        assert_eq!(reader.timeout_ms, 2000);
    }

    #[test]
    fn test_is_success() {
        assert!(ResponseReader::is_success("OK"));
        assert!(ResponseReader::is_success("ok"));
        assert!(ResponseReader::is_success("OK00"));
        assert!(ResponseReader::is_success("0"));
        assert!(ResponseReader::is_success("20"));

        assert!(!ResponseReader::is_success("ER1"));
        assert!(!ResponseReader::is_success("ERROR"));
    }

    #[test]
    fn test_parse_error() {
        assert_eq!(ResponseReader::parse_error("ER1"), Some("ER1".to_string()));
        assert_eq!(
            ResponseReader::parse_error("ER25"),
            Some("ER25".to_string())
        );
        assert_eq!(
            ResponseReader::parse_error("ERROR: timeout"),
            Some("ERROR: timeout".to_string())
        );
        assert_eq!(ResponseReader::parse_error("OK"), None);
    }
}
