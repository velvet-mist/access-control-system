pub struct Config {
    pub backend_url: String,
    pub adapter_token: String,
    pub machine_id: String,
}

impl Config {
    pub fn load() -> Self {
        Self {
            backend_url: "http://127.0.0.1:8000".to_string(),
            adapter_token: "done".to_string(),
            machine_id: "MACHINE_1".to_string(),
        }
    }
}
