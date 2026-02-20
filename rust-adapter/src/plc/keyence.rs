use crate::config::Config;
use crate::error::AdapterError;
use crate::plc::PlcDevice;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct KeyencePlc {
    port: Arc<Mutex<Option<Box<dyn serialport::SerialPort + Send>>>>,
    config: Config,
}

impl KeyencePlc {
    pub fn new(cfg: &Config) -> Self {
        Self {
            port: Arc::new(Mutex::new(None)),
            config: cfg.clone(),
        }
    }

    fn open_port(&mut self) -> Result<(), AdapterError> {
        let mut port_guard = self.port.lock().map_err(|_| AdapterError::Plc)?;
        if port_guard.is_some() {
            return Ok(());
        }

        let port = serialport::new(&self.config.plc_port, self.config.plc_baudrate)
            .timeout(Duration::from_secs(1))
            .open()
            .map_err(|e| AdapterError::Serial(e.to_string()))?;

        *port_guard = Some(port);
        Ok(())
    }

    fn write_to_plc(&mut self, register: u16, value: u16) -> Result<(), AdapterError> {
        self.open_port()?;

        // Modbus RTU function code 0x06 (Write Single Register)
        let slave_addr = self.config.plc_slave_addr;
        let func_code: u8 = 0x06;

        // Build Modbus RTU request frame
        let mut frame: Vec<u8> = Vec::with_capacity(8);
        frame.push(slave_addr);
        frame.push(func_code);
        frame.push((register >> 8) as u8); // Register high byte
        frame.push((register & 0xFF) as u8); // Register low byte
        frame.push((value >> 8) as u8); // Value high byte
        frame.push((value & 0xFF) as u8); // Value low byte

        // Calculate CRC16
        let crc = Self::calculate_crc(&frame);
        frame.push(crc as u8);
        frame.push((crc >> 8) as u8);

        // Write to serial port
        let mut port_guard = self.port.lock().map_err(|_| AdapterError::Plc)?;
        let port = port_guard.as_mut().unwrap();
        port.write_all(&frame)
            .map_err(|e| AdapterError::PlcComm(e.to_string()))?;

        // Small delay for PLC to process
        std::thread::sleep(Duration::from_millis(50));

        Ok(())
    }

    fn calculate_crc(data: &[u8]) -> u16 {
        let mut crc: u16 = 0xFFFF;
        for &byte in data {
            crc ^= byte as u16;
            for _ in 0..8 {
                if (crc & 0x0001) != 0 {
                    crc >>= 1;
                    crc ^= 0xA001;
                } else {
                    crc >>= 1;
                }
            }
        }
        crc
    }
}

impl PlcDevice for KeyencePlc {
    fn set_allow(&mut self) -> Result<(), AdapterError> {
        println!(
            "PLC: ACCESS ALLOWED - Writing to register {}",
            self.config.plc_register_allow
        );
        self.write_to_plc(self.config.plc_register_allow, 1)
    }

    fn set_deny(&mut self) -> Result<(), AdapterError> {
        println!(
            "PLC: ACCESS DENIED - Writing to register {}",
            self.config.plc_register_deny
        );
        self.write_to_plc(self.config.plc_register_deny, 1)
    }

    fn reset_signals(&mut self) -> Result<(), AdapterError> {
        self.write_to_plc(self.config.plc_register_allow, 0)?;
        self.write_to_plc(self.config.plc_register_deny, 0)
    }
}
