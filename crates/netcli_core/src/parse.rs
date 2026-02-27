use std::collections::HashMap;
use std::fmt;

use crate::registry;

#[derive(Debug)]
pub enum ParseError {
    InvalidInput(&'static str),
    TemplateNotFound {
        platform: String,
        command_key: String,
    },
    TemplateInvalid(String),
    EngineError(String),
}

impl ParseError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidInput(_) => "INVALID_INPUT",
            Self::TemplateNotFound { .. } => "TEMPLATE_NOT_FOUND",
            Self::TemplateInvalid(_) => "TEMPLATE_INVALID",
            Self::EngineError(_) => "PARSE_ERROR",
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(field) => write!(f, "required input is empty: {field}"),
            Self::TemplateNotFound {
                platform,
                command_key,
            } => write!(f, "no template for ({platform}, {command_key})"),
            Self::TemplateInvalid(msg) => write!(f, "template compilation failed: {msg}"),
            Self::EngineError(msg) => write!(f, "parse error: {msg}"),
        }
    }
}

impl std::error::Error for ParseError {}

pub fn parse_records(
    platform: &str,
    command_key: &str,
    output_text: &str,
) -> Result<Vec<HashMap<String, String>>, ParseError> {
    if platform.is_empty() {
        return Err(ParseError::InvalidInput("platform"));
    }
    if command_key.is_empty() {
        return Err(ParseError::InvalidInput("command_key"));
    }
    if output_text.is_empty() {
        return Err(ParseError::InvalidInput("output_text"));
    }

    let entry = registry::lookup(platform, command_key).ok_or_else(|| {
        ParseError::TemplateNotFound {
            platform: platform.into(),
            command_key: command_key.into(),
        }
    })?;

    let template_text = registry::load_template_text(entry).ok_or_else(|| {
        ParseError::TemplateNotFound {
            platform: platform.into(),
            command_key: command_key.into(),
        }
    })?;

    let template = textfsm_core::Template::parse_str(template_text)
        .map_err(|e| ParseError::TemplateInvalid(e.to_string()))?;

    let mut parser = template.parser();

    parser
        .parse_text_to_dicts(output_text)
        .map_err(|e| ParseError::EngineError(e.to_string()))
}

pub fn parse_json(platform: &str, command_key: &str, output_text: &str) -> String {
    match parse_records(platform, command_key, output_text) {
        Ok(records) => {
            let records_json =
                serde_json::to_value(&records).unwrap_or(serde_json::Value::Array(vec![]));

            serde_json::json!({
                "ok": true,
                "platform": platform,
                "commandKey": command_key,
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

pub fn parse_command_records(
    platform: &str,
    command: &str,
    output_text: &str,
) -> Result<Vec<HashMap<String, String>>, ParseError> {
    let key = registry::normalize_command(command);
    parse_records(platform, &key, output_text)
}

pub fn parse_command_json(platform: &str, command: &str, output_text: &str) -> String {
    let key = registry::normalize_command(command);
    parse_json(platform, &key, output_text)
}
