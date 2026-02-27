use std::collections::HashMap;

use crate::commands::CommandKey;
use crate::platform::Platform;

/// Errors that can arise during parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    UnknownPlatform(String),
    UnknownCommand(String),
    EmptyInput(&'static str),
    TemplateNotFound(String),
    EngineError(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownPlatform(s) => write!(f, "unknown platform: {s}"),
            Self::UnknownCommand(s) => write!(f, "unknown command key: {s}"),
            Self::EmptyInput(field) => write!(f, "required input is empty: {field}"),
            Self::TemplateNotFound(path) => write!(f, "template not found: {path}"),
            Self::EngineError(msg) => write!(f, "engine error: {msg}"),
        }
    }
}

impl std::error::Error for ParseError {}

impl ParseError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnknownPlatform(_) => "UNKNOWN_PLATFORM",
            Self::UnknownCommand(_) => "UNKNOWN_COMMAND",
            Self::EmptyInput(_) => "INVALID_INPUT",
            Self::TemplateNotFound(_) => "TEMPLATE_NOT_FOUND",
            Self::EngineError(_) => "ENGINE_ERROR",
        }
    }
}

/// Parse raw CLI output into structured records.
///
/// Phase 1: validates inputs and returns an empty `Vec` (stub).
/// Phase 2: will look up the template via [`crate::registry`] and run the
/// TextFSM engine.
pub fn parse_records(
    platform: &str,
    command_key: &str,
    output_text: &str,
) -> Result<Vec<HashMap<String, String>>, ParseError> {
    if platform.is_empty() {
        return Err(ParseError::EmptyInput("platform"));
    }
    if command_key.is_empty() {
        return Err(ParseError::EmptyInput("command_key"));
    }
    if output_text.is_empty() {
        return Err(ParseError::EmptyInput("output_text"));
    }

    let _platform: Platform = platform
        .parse()
        .map_err(|_| ParseError::UnknownPlatform(platform.to_string()))?;
    let _command_key: CommandKey = command_key
        .parse()
        .map_err(|_| ParseError::UnknownCommand(command_key.to_string()))?;

    // TODO (phase 2): look up template via registry, run TextFSM engine
    Ok(Vec::new())
}

/// Convenience wrapper that returns a JSON envelope string.
///
/// Success → `{"ok":true,"platform":"...","commandKey":"...","records":[...]}`
/// Error   → `{"ok":false,"error":{"code":"...","message":"..."}}`
pub fn parse_json(platform: &str, command_key: &str, output_text: &str) -> String {
    match parse_records(platform, command_key, output_text) {
        Ok(records) => {
            let resolved_platform = platform
                .parse::<Platform>()
                .map(|p| p.slug().to_string())
                .unwrap_or_else(|_| platform.to_string());

            let resolved_command = command_key
                .parse::<CommandKey>()
                .map(|c| c.slug().to_string())
                .unwrap_or_else(|_| command_key.to_string());

            let records_json: Vec<serde_json::Value> = records
                .into_iter()
                .map(|r| serde_json::Value::Object(r.into_iter().map(|(k, v)| (k, serde_json::Value::String(v))).collect()))
                .collect();

            serde_json::json!({
                "ok": true,
                "platform": resolved_platform,
                "commandKey": resolved_command,
                "records": records_json,
            })
            .to_string()
        }
        Err(e) => serde_json::json!({
            "ok": false,
            "error": {
                "code": e.code(),
                "message": e.to_string(),
            }
        })
        .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_platform_returns_invalid_input() {
        let err = parse_records("", "show_version", "some output").unwrap_err();
        assert_eq!(err, ParseError::EmptyInput("platform"));
    }

    #[test]
    fn empty_command_key_returns_invalid_input() {
        let err = parse_records("cisco_ios", "", "some output").unwrap_err();
        assert_eq!(err, ParseError::EmptyInput("command_key"));
    }

    #[test]
    fn empty_output_text_returns_invalid_input() {
        let err = parse_records("cisco_ios", "show_version", "").unwrap_err();
        assert_eq!(err, ParseError::EmptyInput("output_text"));
    }

    #[test]
    fn unknown_platform_is_rejected() {
        let err = parse_records("unknown_os", "show_version", "text").unwrap_err();
        assert_eq!(err.code(), "UNKNOWN_PLATFORM");
    }

    #[test]
    fn unknown_command_is_rejected() {
        let err = parse_records("cisco_ios", "show_magic", "text").unwrap_err();
        assert_eq!(err.code(), "UNKNOWN_COMMAND");
    }

    #[test]
    fn valid_inputs_return_empty_records_stub() {
        let records = parse_records("cisco_ios", "show_version", "some output").unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn json_success_envelope_shape() {
        let json_str = parse_json("cisco_ios", "show_version", "some output");
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["ok"], true);
        assert_eq!(v["platform"], "cisco_ios");
        assert_eq!(v["commandKey"], "show_version");
        assert!(v["records"].is_array());
    }

    #[test]
    fn json_error_envelope_shape() {
        let json_str = parse_json("", "show_version", "some output");
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["ok"], false);
        assert!(v["error"]["code"].is_string());
        assert!(v["error"]["message"].is_string());
    }
}
