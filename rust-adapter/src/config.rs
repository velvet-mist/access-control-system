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
    pub plc_type: String,
    pub plc_port: String,
    pub plc_baudrate: u32,
    pub plc_slave_addr: u8,
    pub plc_register_allow: u16,
    pub plc_register_deny: u16,
    // Cognex settings (TCP command scaffold)
    pub cognex_host: String,
    pub cognex_port: u16,
    pub cognex_allow_command: String,
    pub cognex_deny_command: String,
    pub cognex_reset_command: String,
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
            plc_type: env_or("PLC_TYPE", "keyence").to_ascii_lowercase(),
            plc_port: env_or("PLC_PORT", default_serial_port()),
            plc_baudrate: env_parse_or("PLC_BAUDRATE", 9600),
            plc_slave_addr: env_parse_or("PLC_SLAVE_ADDR", 1),
            plc_register_allow: env_parse_or("PLC_REGISTER_ALLOW", 100),
            plc_register_deny: env_parse_or("PLC_REGISTER_DENY", 101),
            cognex_host: env_or("COGNEX_HOST", "127.0.0.1"),
            cognex_port: env_parse_or("COGNEX_PORT", 23),
            cognex_allow_command: env_or("COGNEX_ALLOW_COMMAND", "ALLOW"),
            cognex_deny_command: env_or("COGNEX_DENY_COMMAND", "DENY"),
            cognex_reset_command: env_or("COGNEX_RESET_COMMAND", "RESET"),
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
