use crate::error::AdapterError;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::timeout;

const CONNECT_TIMEOUT:  Duration = Duration::from_secs(5);
const READ_TIMEOUT:     Duration = Duration::from_secs(3);
const RECONNECT_DELAY:  Duration = Duration::from_secs(2);
const MAX_RETRIES:      usize    = 3;

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

    /// Ensure we have a live socket, (re)connecting if necessary.
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

    /// Send `command\r\n` and return the response line (without CRLF).
    /// Retries up to MAX_RETRIES times, reconnecting between attempts.
    pub async fn send_command(&mut self, command: &str) -> Result<String, AdapterError> {
        let mut last_err = AdapterError::PlcComm("no attempts made".to_string());

        for attempt in 1..=MAX_RETRIES {
            if let Err(e) = self.ensure_connected().await {
                last_err = e;
                tokio::time::sleep(RECONNECT_DELAY).await;
                continue;
            }

            match self.try_send_command(command).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    eprintln!(
                        "Keyence command '{}' failed (attempt {}/{}): {}",
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

    async fn try_send_command(&mut self, command: &str) -> Result<String, AdapterError> {
        let inner = self
            .stream
            .as_mut()
            .ok_or_else(|| AdapterError::PlcComm("not connected".to_string()))?;

        // Write command + CRLF
        let payload = format!("{}\r\n", command);
        inner
            .writer
            .write_all(payload.as_bytes())
            .await
            .map_err(|e| AdapterError::PlcComm(format!("write error: {}", e)))?;

        // Read one response line with timeout
        let mut line = String::new();
        timeout(READ_TIMEOUT, inner.reader.read_line(&mut line))
            .await
            .map_err(|_| AdapterError::PlcComm("read timeout".to_string()))?
            .map_err(|e| AdapterError::PlcComm(format!("read error: {}", e)))?;

        // Strip trailing CRLF
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