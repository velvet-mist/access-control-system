#![allow(dead_code)]

use crate::connections::connection::KeyenceConnection;
use crate::error::AdapterError;

/// Keyence PLC command types
#[derive(Debug, Clone, Copy)]
pub enum KeyenceCommand {
    /// Trigger all outputs
    TriggerAll,
    /// Run mode - normal operation
    RunMode,
    /// Setup mode - configuration
    SetupMode,
    /// Test mode
    TestMode,
    /// Reset command
    Reset,
    /// Status query
    Status,
}

impl KeyenceCommand {
    /// Convert command to string representation
    pub fn to_string(&self) -> &'static str {
        match self {
            KeyenceCommand::TriggerAll => "TA",
            KeyenceCommand::RunMode => "R0",
            KeyenceCommand::SetupMode => "S0",
            KeyenceCommand::TestMode => "TM",
            KeyenceCommand::Reset => "RS",
            KeyenceCommand::Status => "ST",
        }
    }

    /// Check if this command expects a response
    pub fn expects_response(&self) -> bool {
        matches!(self, KeyenceCommand::Status)
    }
}

impl std::fmt::Display for KeyenceCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Commander trait for sending commands to Keyence PLC
pub trait Commander {
    fn send_command(&mut self, cmd: KeyenceCommand) -> Result<(), AdapterError>;
}

impl Commander for KeyenceConnection {
    fn send_command(&mut self, cmd: KeyenceCommand) -> Result<(), AdapterError> {
        self.send_command(cmd.to_string())
    }
}

/// Keyence commander with helper methods
pub struct KeyenceCommander;

impl KeyenceCommander {
    /// Create a new Keyence commander
    pub fn new() -> Self {
        Self
    }

    /// Send trigger all command
    pub fn trigger_all(conn: &mut KeyenceConnection) -> Result<(), AdapterError> {
        conn.send_command(KeyenceCommand::TriggerAll.to_string())
    }

    /// Enter run mode
    pub fn run_mode(conn: &mut KeyenceConnection) -> Result<(), AdapterError> {
        conn.send_command(KeyenceCommand::RunMode.to_string())
    }

    /// Enter setup mode
    pub fn setup_mode(conn: &mut KeyenceConnection) -> Result<(), AdapterError> {
        conn.send_command(KeyenceCommand::SetupMode.to_string())
    }

    /// Enter test mode
    pub fn test_mode(conn: &mut KeyenceConnection) -> Result<(), AdapterError> {
        conn.send_command(KeyenceCommand::TestMode.to_string())
    }

    /// Reset the PLC
    pub fn reset(conn: &mut KeyenceConnection) -> Result<(), AdapterError> {
        conn.send_command(KeyenceCommand::Reset.to_string())
    }

    /// Get status from PLC
    pub fn status(conn: &mut KeyenceConnection) -> Result<(), AdapterError> {
        conn.send_command(KeyenceCommand::Status.to_string())
    }

    /// Send custom command string
    pub fn custom(conn: &mut KeyenceConnection, command: &str) -> Result<(), AdapterError> {
        conn.send_command(command)
    }
}

impl Default for KeyenceCommander {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_to_string() {
        assert_eq!(KeyenceCommand::TriggerAll.to_string(), "TA");
        assert_eq!(KeyenceCommand::RunMode.to_string(), "R0");
        assert_eq!(KeyenceCommand::SetupMode.to_string(), "S0");
        assert_eq!(KeyenceCommand::TestMode.to_string(), "TM");
        assert_eq!(KeyenceCommand::Reset.to_string(), "RS");
        assert_eq!(KeyenceCommand::Status.to_string(), "ST");
    }

    #[test]
    fn test_command_display() {
        assert_eq!(format!("{}", KeyenceCommand::TriggerAll), "TA");
    }

    #[test]
    fn test_expects_response() {
        assert!(!KeyenceCommand::TriggerAll.expects_response());
        assert!(!KeyenceCommand::RunMode.expects_response());
        assert!(KeyenceCommand::Status.expects_response());
    }
}
