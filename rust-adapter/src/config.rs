pub struct Config {
    pub backend_url: String,
    pub adapter_token: String,
    pub override_token: String,
    pub override_passcode: String,
    pub machine_id: String,
    // HTTP Server settings
    pub server_host: String,
    pub server_port: u16,
    // Keyence PLC settings
    pub plc_port: String,
    pub plc_baudrate: u32,
    pub plc_slave_addr: u8,
    pub plc_register_allow: u16,
    pub plc_register_deny: u16,
}

impl Config {
    pub fn load() -> Self {
        Self {
            backend_url: "http://127.0.0.1:8000".to_string(),
            adapter_token: "done".to_string(),
            override_token: "override-token".to_string(),
            override_passcode: "1234".to_string(),
            machine_id: "MACHINE_1".to_string(),
            server_host: "0.0.0.0".to_string(),
            server_port: 8080,
            plc_port: "/dev/ttyUSB0".to_string(),
            plc_baudrate: 9600,
            plc_slave_addr: 1,
            plc_register_allow: 100,
            plc_register_deny: 101,
        }
    }
}
