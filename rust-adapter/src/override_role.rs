use crate::config::Config;

pub fn token_matches(config: &Config, provided_token: Option<&str>) -> bool {
    let expected = config.override_token.trim();
    if expected.is_empty() {
        return false;
    }

    provided_token
        .map(str::trim)
        .map(|token| token == expected)
        .unwrap_or(false)
}

pub fn passcode_matches(config: &Config, provided_passcode: Option<&str>) -> bool {
    let expected = config.override_passcode.trim();
    if expected.is_empty() {
        return false;
    }

    provided_passcode
        .map(str::trim)
        .map(|passcode| passcode == expected)
        .unwrap_or(false)
}

pub fn is_override_authorized(
    config: &Config,
    provided_token: Option<&str>,
    provided_passcode: Option<&str>,
) -> bool {
    token_matches(config, provided_token) || passcode_matches(config, provided_passcode)
}
