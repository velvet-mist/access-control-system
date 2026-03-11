use std::env;

#[derive(Clone)]
pub struct Config {
    pub backend_url: String,
    pub adapter_token: String,
    pub override_token: String,
    pub override_passcode: String,
    pub machine_id: String,
    // HTTP Server settings
    pub server_host: String,
    pub server_port: u16,
    // PLC selection/settings
    pub plc_port: String,
    pub plc_baudrate: u32,
    pub plc_slave_addr: u8,
    pub plc_register_request_pending: u16,
    pub plc_request_pending_min_ms: u64,
    pub plc_register_allow: u16,
    pub plc_register_deny: u16,
    pub keyence_host: String,
    pub keyence_port: u16,
    // Embedded Python settings
    pub run_embedded_python: bool,
    pub python_module: String,
    pub python_function: String,
}

impl Config {
    pub fn load() -> Self {
        Self {
            backend_url: env_or("BACKEND_URL", "http://127.0.0.1:8000"),
            adapter_token: env_or("ADAPTER_TOKEN", "done"),
            override_token: env_or("OVERRIDE_TOKEN", "override-token"),
            override_passcode: env_or("OVERRIDE_PASSCODE", "1234"),
            machine_id: env_or("MACHINE_ID", "MACHINE_1"),
            server_host: env_or("SERVER_HOST", "0.0.0.0"),
            server_port: env_parse_or("SERVER_PORT", 8080),
            plc_port: env_or("PLC_PORT", default_serial_port()),
            plc_baudrate: env_parse_or("PLC_BAUDRATE", 9600),
            plc_slave_addr: env_parse_or("PLC_SLAVE_ADDR", 1),
            plc_register_request_pending: env_parse_or("PLC_REGISTER_REQUEST_PENDING", 102),
            plc_request_pending_min_ms: env_parse_or("PLC_REQUEST_PENDING_MIN_MS", 1500),
            plc_register_allow: env_parse_or("PLC_REGISTER_ALLOW", 100),
            plc_register_deny: env_parse_or("PLC_REGISTER_DENY", 101),
            keyence_host: env_or("KEYENCE_HOST", "127.0.0.1"),
            keyence_port: env_parse_or("KEYENCE_PORT", 9004),
            run_embedded_python: env_bool_or("RUN_EMBEDDED_PYTHON", false),
            python_module: env_or("PYTHON_MODULE", "controller.main"),
            python_function: env_or("PYTHON_FUNCTION", "start_application"),
        }
    }
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_parse_or<T>(key: &str, default: T) -> T
where
    T: std::str::FromStr,
{
    env::var(key)
        .ok()
        .and_then(|v| v.parse::<T>().ok())
        .unwrap_or(default)
}

fn env_bool_or(key: &str, default: bool) -> bool {
    match env::var(key) {
        Ok(v) => matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"),
        Err(_) => default,
    }
}

fn default_serial_port() -> &'static str {
    if cfg!(windows) {
        "COM3"
    } else {
        "/dev/ttyUSB0"
    }
}
