use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};

async fn connect_to_camera(ip: &str, port: u16) -> Result<TcpStream, Box<dyn std::error::Error>> {
    let address = format!("172.026.048.092");
    let stream = TcpStream::connect(address).await?;
    println!("Connected to CV-X400!");
    Ok(stream)
}

async fn send_command(stream: &mut TcpStream, command: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Send command
    stream.write_all(command.as_bytes()).await?;
    
    // Read response
    let mut buffer = vec![0u8; 1024];
    let n = stream.read(&mut buffer).await?;
    let response = String::from_utf8_lossy(&buffer[..n]).to_string();
    
    Ok(response)
}