mod config;
mod state;
mod backend;
mod plc;
mod error;
mod api;
mod override_role;

use config::Config;
use plc::keyence::KeyencePlc;
use api::start_server;
// #[warn(unused_imports)]
// use crate::api::CheckAccessRequest;

#[tokio::main]
// async fn check_access(
//     Json(req): Json<CheckAccessRequest>,
// ) -> Result<Json<AccessResponse>, Error> {

//     let decision = backend
//         .check_access(&req.machine_id, &req.card_id)
//         .await?;

//     Ok(Json(AccessResponse {
//         decision,
//     }))
// }

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
