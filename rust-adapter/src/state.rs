use crate::{backend::client::BackendClient, config::Config, error::AdapterError, plc::create_plc_device};

#[allow(dead_code)]
pub async fn run(cfg: Config) -> Result<(), AdapterError> {
    let backend = BackendClient::new(&cfg);
    let plc = create_plc_device(&cfg)?;

    println!("Adapter running (IDLE)");
    println!("PLC type configured: {}", cfg.plc_type);

    let card_id = "CARD123";
    let command = "START";

    let allowed = backend
        .check_access(card_id, &cfg.machine_id, command)
        .await
        .unwrap_or(false);

    let mut plc_guard = plc.lock().map_err(|_| AdapterError::Plc)?;
    if allowed {
        plc_guard.set_allow()?;
    } else {
        plc_guard.set_deny()?;
    }
    Ok(())
}
