use serde_json::Value;

fn parse_value(s: &str) -> Value {
    serde_json::from_str(s).expect("invalid JSON")
}

#[test]
fn cisco_ios_show_version_returns_success_stub() {
    let output = include_str!("fixtures/cisco_ios/show_version.txt");
    let json_str = netcli_core::parse_json("cisco_ios", "show_version", output);
    let v = parse_value(&json_str);

    assert_eq!(v["ok"], true);
    assert_eq!(v["platform"], "cisco_ios");
    assert_eq!(v["commandKey"], "show_version");
    assert!(v["records"].is_array());
}

#[test]
fn juniper_junos_show_version_returns_success_stub() {
    let output = include_str!("fixtures/juniper_junos/show_version.txt");
    let json_str = netcli_core::parse_json("juniper_junos", "show_version", output);
    let v = parse_value(&json_str);

    assert_eq!(v["ok"], true);
    assert_eq!(v["platform"], "juniper_junos");
    assert_eq!(v["commandKey"], "show_version");
    assert!(v["records"].is_array());
}

#[test]
fn alias_platform_resolves_in_envelope() {
    let json_str = netcli_core::parse_json("ios", "show_version", "some output");
    let v = parse_value(&json_str);

    assert_eq!(v["ok"], true);
    assert_eq!(v["platform"], "cisco_ios");
}

#[test]
fn empty_platform_returns_error_envelope() {
    let json_str = netcli_core::parse_json("", "show_version", "text");
    let v = parse_value(&json_str);

    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "INVALID_INPUT");
}

#[test]
fn unknown_platform_returns_error_envelope() {
    let json_str = netcli_core::parse_json("nonexistent_os", "show_version", "text");
    let v = parse_value(&json_str);

    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "UNKNOWN_PLATFORM");
}

#[test]
fn unknown_command_returns_error_envelope() {
    let json_str = netcli_core::parse_json("cisco_ios", "show_magic_unicorn", "text");
    let v = parse_value(&json_str);

    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "UNKNOWN_COMMAND");
}
