mod config;
mod state;
mod backend;
mod plc;
mod error;

use panic_halt as _;
use config::Config;

#[tokio::main]
async fn main() {
    let cfg = Config::load();

    println!("Starting Rust adapter");

    if let Err(e) = state::run(cfg).await {
        eprintln!("Fatal adapter error: {}", e);
    }
}
