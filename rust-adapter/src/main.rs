mod api;
mod backend;
mod config;
mod connections;
mod error;
mod override_role;
mod plc;
mod state;

use api::start_server;
use config::Config;
use plc::create_plc_device;
use std::env;
#[cfg(feature = "embedded-python")]
use pyo3::prelude::*;
#[cfg(feature = "embedded-python")]
use pyo3::types::PyModule;

#[tokio::main]
async fn main() {
    // Load environment variables from .env in cwd, and fallback to project root.
    let _ = dotenvy::dotenv();
    if env::var("BACKEND_URL").is_err() {
        let _ = dotenvy::from_filename("../.env");
    }

    let cfg = Config::load();

    println!("Starting Rust adapter");
    println!("Backend URL: {}", cfg.backend_url);
    println!("Machine ID: {}", cfg.machine_id);
    println!("PLC Type: keyence");
    println!("PLC Port: {} @ {} baud", cfg.plc_port, cfg.plc_baudrate);
    println!("HTTP Server: {}:{}", cfg.server_host, cfg.server_port);

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

    if let Err(e) = start_server(cfg, plc).await {
        eprintln!("Server error: {}", e);
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
