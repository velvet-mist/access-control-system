mod api;
mod backend;
mod config;
mod connections;
mod error;
mod override_role;
mod plc;
mod tcp_handler;

use api::start_server;
use config::Config;
use connections::shared::SharedKeyence;
use plc::create_plc_device;
use tcp_handler::tcp::{new_access_state, start_tcp_proxy};
#[cfg(feature = "embedded-python")]
use pyo3::prelude::*;
#[cfg(feature = "embedded-python")]
use pyo3::types::PyModule;
use std::collections::HashMap;
use std::fs;
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let shell_env = capture_env(&[
        "BACKEND_URL",
        "MACHINE_ID",
        "SERVER_HOST",
        "SERVER_PORT",
        "TCP_PROXY_HOST",
        "TCP_PROXY_PORT",
        "KEYENCE_HOST",
        "KEYENCE_PORT",
        "PLC_HOST",
        "PLC_TCP_PORT",
        "PLC_PORT",
        "PLC_BAUDRATE",
        "PLC_SLAVE_ADDR",
    ]);

    // Load local env first, then fill any missing values from the project root env.
    let _ = dotenvy::dotenv();
    let _ = dotenvy::from_filename("../.env");

    let local_env = load_env_file(".env");
    let root_env = load_env_file("../.env");
    let cfg = Config::load();

    println!("Starting Rust adapter");
    println!("Backend URL: {}", cfg.backend_url);
    println!("Machine ID: {}", cfg.machine_id);
    println!("PLC Type: keyence");
    if cfg.uses_plc_tcp() {
        println!("PLC Transport: TCP {}:{}", cfg.keyence_host, cfg.keyence_port);
    } else {
        println!("PLC Transport: Serial {} @ {} baud", cfg.plc_port, cfg.plc_baudrate);
    }
    println!("Keyence TCP: {}:{}", cfg.keyence_host, cfg.keyence_port);
    println!("HTTP Server: {}:{}", cfg.server_host, cfg.server_port);
    if let Some(port) = cfg.tcp_proxy_port {
        println!("TCP Proxy: {}:{}", cfg.tcp_proxy_host, port);
    } else {
        println!("TCP Proxy: disabled");
    }
    log_config_sources(&shell_env, &local_env, &root_env);
    log_startup_diagnostics(&cfg);

    if cfg.run_embedded_python {
        #[cfg(feature = "embedded-python")]
        {
            let py_module = cfg.python_module.clone();
            let py_function = cfg.python_function.clone();

            tokio::task::spawn_blocking(move || {
                if let Err(e) = run_python(&py_module, &py_function) {
                    eprintln!("Embedded Python error: {:?}", e);
                }
            });
        }

        #[cfg(not(feature = "embedded-python"))]
        {
            eprintln!(
                "RUN_EMBEDDED_PYTHON=true but binary was built without feature 'embedded-python'"
            );
        }
    } else {
        println!("Embedded Python disabled (RUN_EMBEDDED_PYTHON=false)");
    }

    let plc = match create_plc_device(&cfg) {
        Ok(plc) => plc,
        Err(e) => {
            eprintln!("PLC initialization failed: {}", e);
            return;
        }
    };

    // ── Startup: put CV-X into run mode ──────────────────────────────────────
    // Send R0 once on boot so the machine always starts in a known safe state.
    // This runs once and does not block the server from starting.
    println!("Startup: sending R0 to put CV-X in run mode...");
    let startup_keyence = SharedKeyence::new(&cfg.keyence_host, cfg.keyence_port);
    match startup_keyence.send("R0").await {
        Ok(response) => println!("Startup R0: OK ({})", response),
        Err(e) => eprintln!("Startup R0 failed (continuing anyway): {}", e),
    }
    // ─────────────────────────────────────────────────────────────────────────

    // Shared access state — granted by the API when a badge scan is approved,
    // consumed (revoked) when Keyence sends R0 or S0.
    let access = new_access_state();

    if cfg.tcp_proxy_enabled() {
        let tcp_cfg = cfg.clone();
        let tcp_access = access.clone();
        tokio::spawn(async move {
            if let Err(err) = start_tcp_proxy(tcp_cfg, tcp_access).await {
                eprintln!("TCP proxy error: {}", err);
            }
        });
    }

    if let Err(e) = start_server(cfg, plc, access).await {
        eprintln!("Server error: {}", e);
    }
}

fn capture_env(keys: &[&str]) -> HashMap<String, String> {
    let mut values = HashMap::new();
    for key in keys {
        if let Ok(value) = std::env::var(key) {
            values.insert((*key).to_string(), value);
        }
    }
    values
}

fn load_env_file(path: &str) -> HashMap<String, String> {
    let mut values = HashMap::new();
    let Ok(content) = fs::read_to_string(path) else {
        return values;
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = trimmed.split_once('=') {
            values.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    values
}

fn log_config_sources(
    shell_env: &HashMap<String, String>,
    local_env: &HashMap<String, String>,
    root_env: &HashMap<String, String>,
) {
    println!("Config sources:");
    for key in [
        "SERVER_PORT",
        "TCP_PROXY_HOST",
        "TCP_PROXY_PORT",
        "KEYENCE_HOST",
        "KEYENCE_PORT",
        "PLC_HOST",
        "PLC_TCP_PORT",
        "PLC_PORT",
    ] {
        let source = if shell_env.contains_key(key) {
            "shell env"
        } else if local_env.contains_key(key) {
            "rust-adapter/.env"
        } else if root_env.contains_key(key) {
            "project .env"
        } else {
            "default"
        };

        match std::env::var(key) {
            Ok(value) => println!("  {}={} ({})", key, value, source),
            Err(_) => println!("  {}=<unset> ({})", key, source),
        }
    }
}

fn log_startup_diagnostics(cfg: &Config) {
    println!("Startup diagnostics:");

    if cfg.uses_plc_tcp() {
        // PLC is using Keyence TCP transport — check keyence_host:keyence_port only.
        // The old plc_host:plc_tcp_port (Modbus :502) check is intentionally skipped
        // because the Keyence unit does not expose Modbus TCP.
        let address = format!("{}:{}", cfg.keyence_host, cfg.keyence_port);
        match address.parse::<SocketAddr>() {
            Ok(socket_addr) => {
                match TcpStream::connect_timeout(&socket_addr, Duration::from_secs(3)) {
                    Ok(_) => println!("  PLC TCP check: OK ({})", address),
                    Err(err) => {
                        eprintln!("  PLC TCP check: FAILED ({}) - {}", address, err);
                        eprintln!("  Cannot reach Keyence unit — aborting startup.");
                        std::process::exit(1);
                    }
                }
            }
            Err(err) => {
                eprintln!("  PLC TCP check: FAILED ({}) - {}", address, err);
                std::process::exit(1);
            }
        }
    } else {
        match serialport::new(&cfg.plc_port, cfg.plc_baudrate)
            .timeout(Duration::from_secs(1))
            .open()
        {
            Ok(_) => println!("  PLC serial check: OK ({})", cfg.plc_port),
            Err(err) => eprintln!("  PLC serial check: FAILED ({}) - {}", cfg.plc_port, err),
        }

        // Also verify Keyence TCP reachability when using serial transport
        let address = format!("{}:{}", cfg.keyence_host, cfg.keyence_port);
        match address.parse::<SocketAddr>() {
            Ok(socket_addr) => {
                match TcpStream::connect_timeout(&socket_addr, Duration::from_secs(3)) {
                    Ok(_) => println!("  Keyence TCP check: OK ({})", address),
                    Err(err) => {
                        eprintln!("  Keyence TCP check: FAILED ({}) - {}", address, err)
                    }
                }
            }
            Err(err) => eprintln!("  Keyence TCP check: FAILED ({}) - {}", address, err),
        }
    }
}

#[cfg(feature = "embedded-python")]
fn run_python(module_name: &str, function_name: &str) -> PyResult<()> {
    pyo3::Python::initialize();

    Python::attach(|py| {
        let module = PyModule::import(py, module_name)?;
        module.getattr(function_name)?.call0()?;
        Ok(())
    })
}