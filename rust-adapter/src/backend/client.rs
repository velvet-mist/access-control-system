use crate::config::Config;
use crate::error::AdapterError;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize)]
struct AccessResponse{
    decision: String,
}
pub struct BackendClient {
    base_url: String,
    token: String,
    http:Client,
}

impl BackendClient{
    pub fn new(cfg:&Config)-> Self{
        let http= Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();

        Self {
            base_url: cfg.backend_url.clone(),
            token: cfg.adapter_token.clone(),
            http,
        } 
    }
    pub async  fn check_access(
        &self,
        card_id: &str,
     machine_id: &str,
    command: &str,)
 -> Result<bool, AdapterError> {
    let url = format!("{}/api/check-access", self.base_url);

    let resp = self
        .http
        .post(url)
        .header("X-Adapter-Token", &self.token)
        .query(&[
            ("card_id", card_id),
            ("machine_id", machine_id),
            ("command", command),
        ])
        .send()
        .await
        .map_err(|_| AdapterError::Timeout)?;

    if !resp.status().is_success() {
        return Err(AdapterError::Backend);
    }

    let body: AccessResponse = resp.json().await.map_err(|_| AdapterError::Backend)?;

    Ok(body.decision == "ALLOW")
}
}