/// Thin response-parsing helpers.
/// The actual send/receive is now handled by KeyenceConnection / SharedKeyence.

/// Returns true if the response indicates success per the Keyence protocol:
///   - Echo of the sent command (e.g. "R0", "S0")
///   - "0" (numeric success code)
///   - "OK" prefix
///   - "20x" status codes
pub fn is_success(response: &str) -> bool {
    let upper = response.trim().to_uppercase();
    upper == "OK"
        || upper.starts_with("OK")
        || upper == "0"
        || upper.starts_with("20")
}

/// Returns Some(error_string) if the response is a Keyence error code.
///   - "03" → controller in state that does not accept mode switching
///   - "22" → unnecessary parameter included
///   - "ER*" / "ERROR*" → generic error
pub fn parse_error(response: &str) -> Option<String> {
    let trimmed = response.trim();
    match trimmed {
        "03" => Some("03: controller cannot accept mode switch right now".to_string()),
        "22" => Some("22: unnecessary parameter in command".to_string()),
        _ if trimmed.to_uppercase().starts_with("ER") => Some(trimmed.to_string()),
        _ if trimmed.to_uppercase().starts_with("ERROR") => Some(trimmed.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_variants() {
        assert!(is_success("OK"));
        assert!(is_success("ok"));
        assert!(is_success("OK00"));
        assert!(is_success("0"));
        assert!(is_success("200"));
        assert!(is_success("R0"));  // echo = success handled upstream
    }

    #[test]
    fn error_variants() {
        assert_eq!(parse_error("ER1"), Some("ER1".to_string()));
        assert_eq!(parse_error("03"), Some("03: controller cannot accept mode switch right now".to_string()));
        assert_eq!(parse_error("22"), Some("22: unnecessary parameter in command".to_string()));
        assert_eq!(parse_error("OK"), None);
        assert_eq!(parse_error("R0"), None);
    }
}