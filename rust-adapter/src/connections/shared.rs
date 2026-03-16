use crate::connections::connection::KeyenceConnection;
use crate::error::AdapterError;
use std::sync::Arc;
use tokio::sync::Mutex;

/// A cloneable handle to a single shared Keyence TCP connection.
/// All TCP proxy client tasks use the same underlying socket.
#[derive(Clone)]
pub struct SharedKeyence(Arc<Mutex<KeyenceConnection>>);

impl SharedKeyence {
    pub fn new(host: &str, port: u16) -> Self {
        Self(Arc::new(Mutex::new(KeyenceConnection::new(host, port))))
    }

    /// Send a command and return the response. Serialises concurrent callers
    /// so only one command is in flight at a time (Keyence expects req/resp pairs).
    pub async fn send(&self, command: &str) -> Result<String, AdapterError> {
        let mut conn = self.0.lock().await;
        conn.send_command(command).await
    }

    pub async fn is_connected(&self) -> bool {
        self.0.lock().await.is_connected()
    }
}