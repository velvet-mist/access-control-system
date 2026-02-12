mod config;
mod state;
mod backend;
mod plc;
mod error;
mod api;

use config::Config;
use plc::keyence::KeyencePlc;
use api::start_server;

#[tokio::main]
async fn main() {
    let cfg = Config::load();

    println!("Starting Rust adapter for Keyence PLC");
    println!("Backend URL: {}", cfg.backend_url);
    println!("Machine ID: {}", cfg.machine_id);
    println!("PLC Port: {} @ {} baud", cfg.plc_port, cfg.plc_baudrate);
    println!("HTTP Server: {}:{}", cfg.server_host, cfg.server_port);

    // Initialize PLC
    let plc = KeyencePlc::new(&cfg);

    // Start HTTP server (this blocks)
    if let Err(e) = start_server(cfg, plc).await {
        eprintln!("Server error: {}", e);
    }
}
