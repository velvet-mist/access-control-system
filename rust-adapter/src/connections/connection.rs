use crate::error::AdapterError;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::timeout;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const READ_TIMEOUT:    Duration = Duration::from_secs(3);
const RECONNECT_DELAY: Duration = Duration::from_secs(2);
const MAX_RETRIES:     usize    = 3;

/// Keyence execution result codes returned in the response body.
#[derive(Debug, PartialEq)]
pub enum KeyenceResult {
    /// Command executed successfully (echo or "0")
    Ok,
    /// Unnecessary parameter included in command (code 22)
    UnnecessaryParameter,
    /// Command sent during operation that does not accept switching (code 03)
    InvalidState(String),
    /// Unknown response — pass it through as-is
    Unknown(String),
}

impl KeyenceResult {
    /// Parse the response line returned by the Keyence unit.
    ///
    /// Success: unit echoes the command back ("R0", "S0", "0")
    /// Failure: numeric error code ("03", "22")
    pub fn parse(sent_command: &str, response: &str) -> Self {
        let r = response.trim();
        // Echo of the command = success
        if r.eq_ignore_ascii_case(sent_command.trim()) || r == "0" {
            return KeyenceResult::Ok;
        }
        match r {
            "22" => KeyenceResult::UnnecessaryParameter,
            "03" => KeyenceResult::InvalidState(
                "controller is in a mode that does not accept switching".to_string(),
            ),
            other => KeyenceResult::Unknown(other.to_string()),
        }
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, KeyenceResult::Ok)
    }

    pub fn to_adapter_error(&self) -> AdapterError {
        match self {
            KeyenceResult::Ok => unreachable!("not an error"),
            KeyenceResult::UnnecessaryParameter => {
                AdapterError::PlcComm("Keyence error 22: unnecessary parameter".to_string())
            }
            KeyenceResult::InvalidState(msg) => {
                AdapterError::PlcComm(format!("Keyence error 03: {}", msg))
            }
            KeyenceResult::Unknown(raw) => {
                AdapterError::PlcComm(format!("Keyence unknown response: {}", raw))
            }
        }
    }
}

/// Async TCP connection to the Keyence unit.
///
/// Keeps a single persistent connection open. Automatically reconnects
/// on failure so callers never need to manage the socket lifecycle.
pub struct KeyenceConnection {
    host:   String,
    port:   u16,
    stream: Option<Inner>,
}

struct Inner {
    reader: BufReader<tokio::net::tcp::OwnedReadHalf>,
    writer: tokio::net::tcp::OwnedWriteHalf,
}

impl KeyenceConnection {
    /// Create a connection handle. Does not open the socket yet —
    /// the first `send_command` call will establish it.
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            stream: None,
        }
    }

    async fn ensure_connected(&mut self) -> Result<(), AdapterError> {
        if self.stream.is_some() {
            return Ok(());
        }
        self.connect().await
    }

    async fn connect(&mut self) -> Result<(), AdapterError> {
        let addr = format!("{}:{}", self.host, self.port);

        let tcp = timeout(CONNECT_TIMEOUT, TcpStream::connect(&addr))
            .await
            .map_err(|_| AdapterError::PlcComm(format!("connect timeout to {}", addr)))?
            .map_err(|e| AdapterError::PlcComm(format!("connect failed to {}: {}", addr, e)))?;

        // Disable Nagle — we send small command frames and need low latency
        tcp.set_nodelay(true)
            .map_err(|e| AdapterError::PlcComm(e.to_string()))?;

        let (read_half, write_half) = tcp.into_split();
        self.stream = Some(Inner {
            reader: BufReader::new(read_half),
            writer: write_half,
        });

        println!("Keyence TCP connected to {}", addr);
        Ok(())
    }

    fn disconnect(&mut self) {
        if self.stream.take().is_some() {
            println!("Keyence TCP disconnected ({}:{})", self.host, self.port);
        }
    }

    /// Send `command\r\n`, read back the response line, and parse the
    /// Keyence execution result. Returns the raw response string on success.
    /// Retries up to MAX_RETRIES on socket errors.
    pub async fn send_command(&mut self, command: &str) -> Result<String, AdapterError> {
        let mut last_err = AdapterError::PlcComm("no attempts made".to_string());

        for attempt in 1..=MAX_RETRIES {
            if let Err(e) = self.ensure_connected().await {
                last_err = e;
                tokio::time::sleep(RECONNECT_DELAY).await;
                continue;
            }

            match self.try_send(command).await {
                Ok(response) => {
                    let result = KeyenceResult::parse(command, &response);
                    if result.is_ok() {
                        return Ok(response);
                    } else {
                        // Keyence returned an error code — don't retry, surface it
                        return Err(result.to_adapter_error());
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Keyence command '{}' socket error (attempt {}/{}): {}",
                        command, attempt, MAX_RETRIES, e
                    );
                    last_err = e;
                    self.disconnect();
                    if attempt < MAX_RETRIES {
                        tokio::time::sleep(RECONNECT_DELAY).await;
                    }
                }
            }
        }

        Err(last_err)
    }

    async fn try_send(&mut self, command: &str) -> Result<String, AdapterError> {
        let inner = self
            .stream
            .as_mut()
            .ok_or_else(|| AdapterError::PlcComm("not connected".to_string()))?;

        // Send command + CRLF delimiter (per Keyence protocol spec)
        let payload = format!("{}\r\n", command.trim());
        inner
            .writer
            .write_all(payload.as_bytes())
            .await
            .map_err(|e| AdapterError::PlcComm(format!("write error: {}", e)))?;

        // Read one response line with timeout (response is also CRLF terminated)
        let mut line = String::new();
        timeout(READ_TIMEOUT, inner.reader.read_line(&mut line))
            .await
            .map_err(|_| AdapterError::PlcComm("read timeout — Keyence unit not responding".to_string()))?
            .map_err(|e| AdapterError::PlcComm(format!("read error: {}", e)))?;

        // Strip CRLF delimiter
        let response = line
            .trim_end_matches('\n')
            .trim_end_matches('\r')
            .to_string();

        Ok(response)
    }

    /// Returns true if the socket is currently open.
    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r0_echo_is_success() {
        assert_eq!(KeyenceResult::parse("R0", "R0"), KeyenceResult::Ok);
    }

    #[test]
    fn s0_echo_is_success() {
        assert_eq!(KeyenceResult::parse("S0", "S0"), KeyenceResult::Ok);
    }

    #[test]
    fn zero_is_success() {
        assert_eq!(KeyenceResult::parse("R0", "0"), KeyenceResult::Ok);
    }

    #[test]
    fn error_22_parsed() {
        assert_eq!(
            KeyenceResult::parse("R0", "22"),
            KeyenceResult::UnnecessaryParameter
        );
    }

    #[test]
    fn error_03_parsed() {
        assert!(matches!(
            KeyenceResult::parse("R0", "03"),
            KeyenceResult::InvalidState(_)
        ));
    }

    #[test]
    fn case_insensitive_echo() {
        assert_eq!(KeyenceResult::parse("r0", "R0"), KeyenceResult::Ok);
        assert_eq!(KeyenceResult::parse("R0", "r0"), KeyenceResult::Ok);
    }
}