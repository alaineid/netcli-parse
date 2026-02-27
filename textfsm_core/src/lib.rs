use serde_json::{json, Value};

/// Parse device output using a TextFSM template and return a JSON envelope.
///
/// Returns a JSON string — always an object with an `"ok"` field.
/// If any input is empty, returns an `INVALID_INPUT` error envelope.
/// Otherwise returns a success envelope with an empty `records` array (stub).
pub fn parse_json(
    vendor: &str,
    command_key: &str,
    template_text: &str,
    output_text: &str,
) -> String {
    if vendor.is_empty()
        || command_key.is_empty()
        || template_text.is_empty()
        || output_text.is_empty()
    {
        return error_envelope("INVALID_INPUT", "One or more required inputs are empty");
    }

    let envelope: Value = json!({
        "ok": true,
        "vendor": vendor,
        "commandKey": command_key,
        "records": []
    });

    envelope.to_string()
}

fn error_envelope(code: &str, message: &str) -> String {
    let envelope: Value = json!({
        "ok": false,
        "error": {
            "code": code,
            "message": message
        }
    });
    envelope.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn parse_value(s: &str) -> Value {
        serde_json::from_str(s).expect("invalid JSON returned")
    }

    #[test]
    fn success_envelope_has_expected_fields() {
        let json_str = parse_json("cisco_ios", "show version", "Value UPTIME", "router uptime");
        let v = parse_value(&json_str);

        assert_eq!(v["ok"], json!(true));
        assert_eq!(v["vendor"], json!("cisco_ios"));
        assert_eq!(v["commandKey"], json!("show version"));
        assert!(v["records"].is_array());
        assert_eq!(v["records"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn empty_vendor_returns_invalid_input() {
        let json_str = parse_json("", "show version", "Value UPTIME", "router uptime");
        let v = parse_value(&json_str);

        assert_eq!(v["ok"], json!(false));
        assert_eq!(v["error"]["code"], json!("INVALID_INPUT"));
    }

    #[test]
    fn empty_command_key_returns_invalid_input() {
        let json_str = parse_json("cisco_ios", "", "Value UPTIME", "router uptime");
        let v = parse_value(&json_str);

        assert_eq!(v["ok"], json!(false));
        assert_eq!(v["error"]["code"], json!("INVALID_INPUT"));
    }

    #[test]
    fn empty_template_returns_invalid_input() {
        let json_str = parse_json("cisco_ios", "show version", "", "router uptime");
        let v = parse_value(&json_str);

        assert_eq!(v["ok"], json!(false));
        assert_eq!(v["error"]["code"], json!("INVALID_INPUT"));
    }

    #[test]
    fn empty_output_returns_invalid_input() {
        let json_str = parse_json("cisco_ios", "show version", "Value UPTIME", "");
        let v = parse_value(&json_str);

        assert_eq!(v["ok"], json!(false));
        assert_eq!(v["error"]["code"], json!("INVALID_INPUT"));
    }

    #[test]
    fn error_envelope_message_is_present() {
        let json_str = parse_json("", "", "", "");
        let v = parse_value(&json_str);

        assert!(v["error"]["message"].is_string());
        assert!(!v["error"]["message"].as_str().unwrap().is_empty());
    }
}
