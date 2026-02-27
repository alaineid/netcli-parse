use serde_json::Value;

fn parse_envelope(s: &str) -> Value {
    serde_json::from_str(s).expect("invalid JSON envelope")
}

#[test]
fn cisco_ios_show_version_parses_real_output() {
    let output = include_str!("fixtures/cisco_ios/show_version.txt");
    let json_str = netcli_core::parse_json("cisco_ios", "show_version", output);
    let v = parse_envelope(&json_str);

    assert_eq!(v["ok"], true);
    assert_eq!(v["platform"], "cisco_ios");
    assert_eq!(v["commandKey"], "show_version");
    assert!(v["records"].is_array());

    let records = v["records"].as_array().unwrap();
    assert!(!records.is_empty(), "expected at least one parsed record");
}

#[test]
fn juniper_junos_show_version_parses_real_output() {
    let output = include_str!("fixtures/juniper_junos/show_version.txt");
    let json_str = netcli_core::parse_json("juniper_junos", "show_version", output);
    let v = parse_envelope(&json_str);

    assert_eq!(v["ok"], true);
    assert_eq!(v["platform"], "juniper_junos");
    assert_eq!(v["commandKey"], "show_version");

    let records = v["records"].as_array().unwrap();
    assert!(!records.is_empty(), "expected at least one parsed record");
}

#[test]
fn unknown_platform_returns_template_not_found() {
    let json_str = netcli_core::parse_json("nonexistent_os", "show_version", "some output");
    let v = parse_envelope(&json_str);

    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "TEMPLATE_NOT_FOUND");
}

#[test]
fn unknown_command_returns_template_not_found() {
    let json_str = netcli_core::parse_json("cisco_ios", "show_magic_unicorn", "some output");
    let v = parse_envelope(&json_str);

    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "TEMPLATE_NOT_FOUND");
}

#[test]
fn empty_platform_returns_invalid_input() {
    let json_str = netcli_core::parse_json("", "show_version", "some output");
    let v = parse_envelope(&json_str);

    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "INVALID_INPUT");
}

#[test]
fn empty_command_key_returns_invalid_input() {
    let json_str = netcli_core::parse_json("cisco_ios", "", "some output");
    let v = parse_envelope(&json_str);

    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "INVALID_INPUT");
}

#[test]
fn empty_output_text_returns_invalid_input() {
    let json_str = netcli_core::parse_json("cisco_ios", "show_version", "");
    let v = parse_envelope(&json_str);

    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "INVALID_INPUT");
}

// --- prompt tolerance tests ---
// With ^. -> Error removed from all templates, unrecognized lines
// (prompts, banners, etc.) are silently ignored by the TextFSM engine.

#[test]
fn cisco_ios_show_version_tolerates_prompt() {
    let clean = include_str!("fixtures/cisco_ios/show_version.txt");
    let with_prompt = format!("router1#show version\n{clean}\nrouter1#");

    let json_str = netcli_core::parse_json("cisco_ios", "show_version", &with_prompt);
    let v = parse_envelope(&json_str);

    assert_eq!(v["ok"], true, "engine should ignore prompt lines");
    let records = v["records"].as_array().unwrap();
    assert!(!records.is_empty(), "should still parse records despite prompt");
}

#[test]
fn zte_zxros_show_version_tolerates_prompt() {
    // zte_zxros/show_version previously had ^. -> Error — now removed
    let clean = "\
ZXCTN6500\n\
ZTE ZXCTN Software, Version: V4.00.10, Release software\n\
router2 uptime is 5 days\n";

    let with_prompt = format!("router2#show version\n{clean}\nrouter2#");

    let json_str = netcli_core::parse_json("zte_zxros", "show_version", &with_prompt);
    let v = parse_envelope(&json_str);

    assert_eq!(v["ok"], true, "engine should ignore prompt lines now that ^. -> Error is removed: {v}");
    let records = v["records"].as_array().unwrap();
    assert!(!records.is_empty(), "should parse records despite prompt");
}

// --- command-based API tests ---

#[test]
fn command_api_cisco_ios_show_version() {
    let output = include_str!("fixtures/cisco_ios/show_version.txt");
    let json_str = netcli_core::parse_command_json("cisco_ios", "show version", output);
    let v = parse_envelope(&json_str);

    assert_eq!(v["ok"], true);
    assert_eq!(v["platform"], "cisco_ios");
    assert_eq!(v["commandKey"], "show_version");

    let records = v["records"].as_array().unwrap();
    assert!(!records.is_empty(), "expected at least one parsed record");
}

#[test]
fn command_api_matches_key_api() {
    let output = include_str!("fixtures/cisco_ios/show_version.txt");
    let by_key = netcli_core::parse_json("cisco_ios", "show_version", output);
    let by_cmd = netcli_core::parse_command_json("cisco_ios", "show version", output);

    let v_key: Value = parse_envelope(&by_key);
    let v_cmd: Value = parse_envelope(&by_cmd);

    assert_eq!(v_key["records"], v_cmd["records"], "both APIs should produce identical records");
}

#[test]
fn command_api_normalizes_mixed_case_and_extra_spaces() {
    let output = include_str!("fixtures/cisco_ios/show_version.txt");
    let json_str = netcli_core::parse_command_json("cisco_ios", "  Show   Version  ", output);
    let v = parse_envelope(&json_str);

    assert_eq!(v["ok"], true, "normalization should handle mixed case and extra whitespace");
    assert_eq!(v["commandKey"], "show_version");
}

#[test]
fn command_api_empty_command_returns_invalid_input() {
    let json_str = netcli_core::parse_command_json("cisco_ios", "", "some output");
    let v = parse_envelope(&json_str);

    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "INVALID_INPUT");
}

#[test]
fn command_api_whitespace_only_command_returns_invalid_input() {
    let json_str = netcli_core::parse_command_json("cisco_ios", "   ", "some output");
    let v = parse_envelope(&json_str);

    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "INVALID_INPUT");
}
