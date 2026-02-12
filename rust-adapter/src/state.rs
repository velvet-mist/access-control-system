use crate::{
    backend::client::BackendClient,
    config::Config,
    error::AdapterError,
    plc::keyence::KeyencePlc,
};

pub async fn run(cfg: Config) -> Result<(), AdapterError> {
    let backend = BackendClient::new(&cfg);
    let mut plc = KeyencePlc::new(&cfg);

    println!("Adapter running (IDLE)");
    println!("PLC configured on port: {} @ {} baud", cfg.plc_port, cfg.plc_baudrate);

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
