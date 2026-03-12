use crate::config::Config;
use crate::error::AdapterError;
use crate::plc::PlcDevice;
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

enum PlcTransport {
    Serial(Box<dyn serialport::SerialPort + Send>),
    Tcp(TcpStream),
}

pub struct KeyencePlc {
    connection: Arc<Mutex<Option<PlcTransport>>>,
    config: Config,
    transaction_id: u16,
}

impl KeyencePlc {
    pub fn new(cfg: &Config) -> Self {
        Self {
            connection: Arc::new(Mutex::new(None)),
            config: cfg.clone(),
            transaction_id: 0,
        }
    }

    fn open_connection(&mut self) -> Result<(), AdapterError> {
        let mut connection_guard = self.connection.lock().map_err(|_| AdapterError::Plc)?;
        if connection_guard.is_some() {
            return Ok(());
        }

        let connection = if self.config.uses_plc_tcp() {
            let address = format!("{}:{}", self.config.plc_host, self.config.plc_tcp_port);
            let socket_addr: SocketAddr = address
                .parse()
                .map_err(|e: std::net::AddrParseError| AdapterError::PlcComm(e.to_string()))?;
            let stream = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(3))
                .map_err(|e| AdapterError::PlcComm(e.to_string()))?;
            stream
                .set_write_timeout(Some(Duration::from_secs(2)))
                .map_err(|e| AdapterError::PlcComm(e.to_string()))?;
            PlcTransport::Tcp(stream)
        } else {
            let port = serialport::new(&self.config.plc_port, self.config.plc_baudrate)
                .timeout(Duration::from_secs(1))
                .open()
                .map_err(|e| AdapterError::Serial(e.to_string()))?;
            PlcTransport::Serial(port)
        };

        *connection_guard = Some(connection);
        Ok(())
    }

    fn write_to_plc(&mut self, register: u16, value: u16) -> Result<(), AdapterError> {
        self.open_connection()?;

        let mut connection_guard = self.connection.lock().map_err(|_| AdapterError::Plc)?;
        let connection = connection_guard.as_mut().ok_or(AdapterError::Plc)?;

        match connection {
            PlcTransport::Serial(port) => {
                let frame = self.build_modbus_rtu_frame(register, value);
                port.write_all(&frame)
                    .map_err(|e| AdapterError::PlcComm(e.to_string()))?;
            }
            PlcTransport::Tcp(stream) => {
                let frame = self.build_modbus_tcp_frame(register, value);
                stream
                    .write_all(&frame)
                    .map_err(|e| AdapterError::PlcComm(e.to_string()))?;
                self.transaction_id = self.transaction_id.wrapping_add(1);
            }
        }

        std::thread::sleep(Duration::from_millis(50));
        Ok(())
    }

    fn build_modbus_rtu_frame(&self, register: u16, value: u16) -> Vec<u8> {
        let slave_addr = self.config.plc_slave_addr;
        let func_code: u8 = 0x06;

        let mut frame: Vec<u8> = Vec::with_capacity(8);
        frame.push(slave_addr);
        frame.push(func_code);
        frame.push((register >> 8) as u8);
        frame.push((register & 0xFF) as u8);
        frame.push((value >> 8) as u8);
        frame.push((value & 0xFF) as u8);

        let crc = Self::calculate_crc(&frame);
        frame.push(crc as u8);
        frame.push((crc >> 8) as u8);
        frame
    }

    fn build_modbus_tcp_frame(&self, register: u16, value: u16) -> Vec<u8> {
        let transaction_id = self.transaction_id;
        let protocol_id: u16 = 0;
        let unit_id = self.config.plc_slave_addr;
        let pdu_len: u16 = 6;
        let func_code: u8 = 0x06;

        vec![
            (transaction_id >> 8) as u8,
            (transaction_id & 0xFF) as u8,
            (protocol_id >> 8) as u8,
            (protocol_id & 0xFF) as u8,
            (pdu_len >> 8) as u8,
            (pdu_len & 0xFF) as u8,
            unit_id,
            func_code,
            (register >> 8) as u8,
            (register & 0xFF) as u8,
            (value >> 8) as u8,
            (value & 0xFF) as u8,
        ]
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
    fn set_request_pending(&mut self) -> Result<(), AdapterError> {
        println!(
            "PLC: ACCESS REQUEST PENDING - Writing to register {}",
            self.config.plc_register_request_pending
        );
        self.write_to_plc(self.config.plc_register_request_pending, 1)
    }

    fn clear_request_pending(&mut self) -> Result<(), AdapterError> {
        self.write_to_plc(self.config.plc_register_request_pending, 0)
    }

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
        self.clear_request_pending()?;
        self.write_to_plc(self.config.plc_register_allow, 0)?;
        self.write_to_plc(self.config.plc_register_deny, 0)
    }
}
