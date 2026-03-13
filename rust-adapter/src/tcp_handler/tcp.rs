use crate::config::Config;
use crate::connections::connection::KeyenceConnection;
use crate::connections::read::send_and_read;
use crate::error::AdapterError;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

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

async fn forward_command(cfg: Config, command: String) -> Result<String, AdapterError> {
    tokio::task::spawn_blocking(move || {
        let mut conn = KeyenceConnection::new(&cfg.keyence_host, cfg.keyence_port)?;
        send_and_read(&mut conn, &command)
    })
    .await
    .map_err(|err| AdapterError::PlcComm(format!("tcp proxy task failed: {}", err)))?
}

async fn handle_client(stream: TcpStream, cfg: Config) -> Result<(), AdapterError> {
    let peer = stream
        .peer_addr()
        .map(|addr| addr.to_string())
        .unwrap_or_else(|_| "<unknown>".to_string());
    let (reader_half, mut writer_half) = stream.into_split();
    let mut reader = BufReader::new(reader_half);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader
            .read_line(&mut line)
            .await
            .map_err(|err| AdapterError::PlcComm(err.to_string()))?;
        if bytes_read == 0 {
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let command = normalize_command(trimmed);
        println!("TCP proxy {} -> {}", peer, command);

        if is_protected_command(&command) {
            writer_half
                .write_all(b"ER\r\n")
                .await
                .map_err(|err| AdapterError::PlcComm(err.to_string()))?;
            continue;
        }

        match forward_command(cfg.clone(), command).await {
            Ok(response) => {
                let payload = if response.is_empty() {
                    "\r\n".to_string()
                } else {
                    format!("{}\r\n", response)
                };

                writer_half
                    .write_all(payload.as_bytes())
                    .await
                    .map_err(|err| AdapterError::PlcComm(err.to_string()))?;
            }
            Err(err) => {
                eprintln!("TCP proxy forward error for {}: {}", peer, err);
                writer_half
                    .write_all(b"ER\r\n")
                    .await
                    .map_err(|write_err| AdapterError::PlcComm(write_err.to_string()))?;
            }
        }
    }

    Ok(())
}

pub async fn start_tcp_proxy(cfg: Config) -> Result<(), AdapterError> {
    let Some(port) = cfg.tcp_proxy_port else {
        return Ok(());
    };

    let addr = format!("{}:{}", cfg.tcp_proxy_host, port);
    let listener = TcpListener::bind(&addr).await.map_err(|err| {
        AdapterError::PlcComm(format!("tcp proxy bind failed for {}: {}", addr, err))
    })?;

    println!(
        "Starting TCP proxy on {} -> {}:{}",
        addr, cfg.keyence_host, cfg.keyence_port
    );

    loop {
        let (stream, _) = listener
            .accept()
            .await
            .map_err(|err| AdapterError::PlcComm(err.to_string()))?;
        let client_cfg = cfg.clone();

        tokio::spawn(async move {
            if let Err(err) = handle_client(stream, client_cfg).await {
                eprintln!("TCP proxy client error: {}", err);
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
}
