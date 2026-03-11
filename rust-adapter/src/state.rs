use crate::{
    backend::client::BackendClient, config::Config, error::AdapterError, plc::create_plc_device,
};
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[allow(dead_code)]
pub async fn run(cfg: Config) -> Result<(), AdapterError> {
    let backend = BackendClient::new(&cfg);
    let plc = create_plc_device(&cfg)?;

    println!("Adapter running (IDLE)");
    println!("PLC type configured: {}", cfg.plc_type);

    let card_id = "CARD123";
    let command = "START";
    let pending_started_at = Instant::now();

    {
        let mut plc_guard = plc.lock().map_err(|_| AdapterError::Plc)?;
        plc_guard.set_request_pending()?;
    }

    let allowed = backend
        .check_access(card_id, &cfg.machine_id, command)
        .await
        .unwrap_or(false);

    let min_pending = Duration::from_millis(cfg.plc_request_pending_min_ms);
    if pending_started_at.elapsed() < min_pending {
        sleep(min_pending - pending_started_at.elapsed()).await;
    }

    let mut plc_guard = plc.lock().map_err(|_| AdapterError::Plc)?;
    if allowed {
        plc_guard.set_allow()?;
    } else {
        plc_guard.set_deny()?;
    }
    plc_guard.clear_request_pending()?;
    Ok(())
}
