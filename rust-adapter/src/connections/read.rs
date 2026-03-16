use crate::error::AdapterError;

/// Check if response indicates success (typically "OK" or starts with specific code)
pub fn is_success(response: &str) -> bool {
    let upper = response.to_uppercase();
    upper == "OK"
        || upper.starts_with("OK")
        || upper.starts_with("0")
        || upper.starts_with("20")
}

/// Parse error code from response
pub fn parse_error(response: &str) -> Option<String> {
    if response.starts_with("ER") {
        Some(response.to_string())
    } else if response.to_uppercase().starts_with("ERROR") {
        Some(response.to_string())
    } else {
        None
    }
}

/// Convenience function to send command and read response
pub async fn send_and_read(conn: &mut crate::connections::connection::KeyenceConnection, command: &str) -> Result<String, AdapterError> {
    conn.send_command(command).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_success() {
        assert!(is_success("OK"));
        assert!(is_success("ok"));
        assert!(is_success("OK00"));
        assert!(is_success("0"));
        assert!(is_success("20"));

        assert!(!is_success("ER1"));
        assert!(!is_success("ERROR"));
    }

    #[test]
    fn test_parse_error() {
        assert_eq!(parse_error("ER1"), Some("ER1".to_string()));
        assert_eq!(parse_error("ER25"), Some("ER25".to_string()));
        assert_eq!(parse_error("ERROR: timeout"), Some("ERROR: timeout".to_string()));
        assert_eq!(parse_error("OK"), None);
    }
}
