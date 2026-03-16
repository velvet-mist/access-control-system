use crate::config::Config;
use crate::connections::shared::SharedKeyence;
use crate::error::AdapterError;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

/// Shared access state — set to true when the backend approves a badge scan,
/// consumed (revoked) when Keyence sends R0 or S0.
pub type AccessState = Arc<Mutex<bool>>;

pub fn new_access_state() -> AccessState {
    Arc::new(Mutex::new(false))
}

pub fn grant_access(state: &AccessState) {
    if let Ok(mut guard) = state.lock() {
        *guard = true;
    }
}

pub fn revoke_access(state: &AccessState) {
    if let Ok(mut guard) = state.lock() {
        *guard = false;
    }
}

fn is_access_granted(state: &AccessState) -> bool {
    state.lock().map(|g| *g).unwrap_or(false)
}

fn normalize_command(command: &str) -> String {
    match command.trim().to_ascii_uppercase().as_str() {
        "RO" | "R0" => "R0".to_string(),
        "SO" | "S0" => "S0".to_string(),
        other => other.to_string(),
    }
}

fn is_protected_command(command: &str) -> bool {
    matches!(command, "R0" | "S0")
}

async fn handle_client(
    stream: TcpStream,
    keyence: SharedKeyence,
    access: AccessState,
) -> Result<(), AdapterError> {
    let peer = stream
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "<unknown>".to_string());

    let (reader_half, mut writer_half) = stream.into_split();
    let mut reader = BufReader::new(reader_half);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader
            .read_line(&mut line)
            .await
            .map_err(|e| AdapterError::PlcComm(e.to_string()))?;

        if bytes_read == 0 {
            // Client disconnected cleanly
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let command = normalize_command(trimmed);
        println!("TCP proxy {} -> {}", peer, command);

        // Gate R0 and S0 behind access check
        if is_protected_command(&command) {
            if !is_access_granted(&access) {
                println!(
                    "TCP proxy: blocked {} from {} — access not granted",
                    command, peer
                );
                writer_half
                    .write_all(b"ER\r\n")
                    .await
                    .map_err(|e| AdapterError::PlcComm(e.to_string()))?;
                continue;
            }

            // Access granted — consume it immediately (one-shot per badge scan)
            println!(
                "TCP proxy: allowing {} from {} — access granted",
                command, peer
            );
            revoke_access(&access);
        }

        // Forward to Keyence over the shared persistent connection
        match keyence.send(&command).await {
            Ok(response) => {
                println!("TCP proxy {} <- {}", peer, response);
                let payload = if response.is_empty() {
                    "\r\n".to_string()
                } else {
                    format!("{}\r\n", response)
                };
                writer_half
                    .write_all(payload.as_bytes())
                    .await
                    .map_err(|e| AdapterError::PlcComm(e.to_string()))?;
            }
            Err(e) => {
                eprintln!("TCP proxy forward error for {}: {}", peer, e);
                writer_half
                    .write_all(b"ER\r\n")
                    .await
                    .map_err(|e| AdapterError::PlcComm(e.to_string()))?;
            }
        }
    }

    println!("TCP proxy client disconnected: {}", peer);
    Ok(())
}

pub async fn start_tcp_proxy(
    cfg: Config,
    access: AccessState,
) -> Result<(), AdapterError> {
    let Some(port) = cfg.tcp_proxy_port else {
        return Ok(());
    };

    // One shared persistent connection to the Keyence unit for all clients
    let keyence = SharedKeyence::new(&cfg.keyence_host, cfg.keyence_port);

    let addr = format!("{}:{}", cfg.tcp_proxy_host, port);
    let listener = TcpListener::bind(&addr).await.map_err(|e| {
        AdapterError::PlcComm(format!("tcp proxy bind failed for {}: {}", addr, e))
    })?;

    println!(
        "Starting TCP proxy on {} -> {}:{}",
        addr, cfg.keyence_host, cfg.keyence_port
    );

    loop {
        let (stream, _) = listener
            .accept()
            .await
            .map_err(|e| AdapterError::PlcComm(e.to_string()))?;

        let client_keyence = keyence.clone();
        let client_access = Arc::clone(&access);

        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, client_keyence, client_access).await {
                eprintln!("TCP proxy client error: {}", e);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aliases_are_normalized() {
        assert_eq!(normalize_command("ro"), "R0");
        assert_eq!(normalize_command("SO"), "S0");
        assert_eq!(normalize_command("ta"), "TA");
    }

    #[test]
    fn protected_commands_are_detected() {
        assert!(is_protected_command("R0"));
        assert!(is_protected_command("S0"));
        assert!(!is_protected_command("TA"));
    }

    #[test]
    fn access_state_grant_revoke() {
        let state = new_access_state();
        assert!(!is_access_granted(&state));
        grant_access(&state);
        assert!(is_access_granted(&state));
        revoke_access(&state);
        assert!(!is_access_granted(&state));
    }

    #[test]
    fn access_is_one_shot() {
        let state = new_access_state();
        grant_access(&state);
        assert!(is_access_granted(&state));
        revoke_access(&state);
        assert!(!is_access_granted(&state));
    }
}