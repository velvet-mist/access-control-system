use crate::{
    backend::client::BackendClient,
    config::Config,
    error::AdapterError,
    plc::keyence::KeyencePlc,
};

pub async fn run(cfg: Config) -> Result<(), AdapterError> {
    let backend = BackendClient::new(&cfg);
    let plc = KeyencePlc::new();

    println!("Adapter running (IDLE)");

    // ---- simulated event ----
    let card_id = "CARD123";
    let command = "START";

    let allowed = backend
        .check_access(card_id, &cfg.machine_id, command)
        .await
        .unwrap_or(false);

    if allowed {
        plc.set_allow()?;
    } else {
        plc.set_deny()?;
    }

    Ok(())
}
