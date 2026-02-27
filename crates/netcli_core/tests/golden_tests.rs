use serde_json::Value;

fn parse_envelope(s: &str) -> Value {
    serde_json::from_str(s).expect("invalid JSON envelope")
}

fn assert_success(v: &Value) {
    assert_eq!(v["ok"], true, "expected ok:true, got: {v}");
    assert!(v["records"].is_array(), "records should be an array");
}

fn records(v: &Value) -> &Vec<Value> {
    v["records"].as_array().unwrap()
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

// --- additional platform golden tests ---

#[test]
fn arista_eos_show_version_parses_real_output() {
    let output = include_str!("fixtures/arista_eos/show_version.txt");
    let json_str = netcli_core::parse_json("arista_eos", "show_version", output);
    let v = parse_envelope(&json_str);

    assert_success(&v);
    assert_eq!(v["platform"], "arista_eos");

    let recs = records(&v);
    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0]["model"], "DCS-7050TX-64");
    assert_eq!(recs[0]["serial_number"], "JPE14080459");
    assert_eq!(recs[0]["image"], "4.20.1F");
    assert_eq!(recs[0]["sys_mac"], "001c.7300.0001");
}

#[test]
fn cisco_nxos_show_version_parses_real_output() {
    let output = include_str!("fixtures/cisco_nxos/show_version.txt");
    let json_str = netcli_core::parse_json("cisco_nxos", "show_version", output);
    let v = parse_envelope(&json_str);

    assert_success(&v);
    assert_eq!(v["platform"], "cisco_nxos");

    let recs = records(&v);
    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0]["hostname"], "nxos-switch1");
    assert_eq!(recs[0]["platform"], "C93180YC-EX");
    assert_eq!(recs[0]["os"], "9.3(8)");
    assert_eq!(recs[0]["serial"], "FDO21120ABC");
}

#[test]
fn cisco_ios_show_version_field_values() {
    let output = include_str!("fixtures/cisco_ios/show_version.txt");
    let json_str = netcli_core::parse_json("cisco_ios", "show_version", output);
    let v = parse_envelope(&json_str);

    assert_success(&v);
    let recs = records(&v);
    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0]["hostname"], "Router01");
    assert_eq!(recs[0]["version"], "12.2(55)SE10");
    assert_eq!(recs[0]["software_image"], "C3750-IPSERVICESK9-M");
    assert_eq!(recs[0]["running_image"], "c3750-ipservicesk9-mz.122-55.SE10.bin");
}

// --- multi-record parsing tests ---

#[test]
fn cisco_ios_show_interfaces_parses_multiple_records() {
    let output = include_str!("fixtures/cisco_ios/show_interfaces.txt");
    let json_str = netcli_core::parse_json("cisco_ios", "show_interfaces", output);
    let v = parse_envelope(&json_str);

    assert_success(&v);
    let recs = records(&v);
    assert_eq!(recs.len(), 2, "should parse two interfaces");

    assert_eq!(recs[0]["interface"], "GigabitEthernet0/1");
    assert_eq!(recs[0]["link_status"], "up");
    assert_eq!(recs[0]["protocol_status"], "up");
    assert_eq!(recs[0]["ip_address"], "10.0.0.1");
    assert_eq!(recs[0]["description"], "Uplink to Core");
    assert_eq!(recs[0]["mac_address"], "0026.9876.1234");

    assert_eq!(recs[1]["interface"], "GigabitEthernet0/2");
    assert_eq!(recs[1]["link_status"], "administratively down");
    assert_eq!(recs[1]["protocol_status"], "down");
    assert_eq!(recs[1]["ip_address"], "192.168.1.1");
}

#[test]
fn cisco_ios_show_ip_bgp_summary_parses_multiple_neighbors() {
    let output = include_str!("fixtures/cisco_ios/show_ip_bgp_summary.txt");
    let json_str = netcli_core::parse_json("cisco_ios", "show_ip_bgp_summary", output);
    let v = parse_envelope(&json_str);

    assert_success(&v);
    let recs = records(&v);
    assert!(recs.len() >= 2, "should parse at least two BGP neighbors, got {}", recs.len());

    assert_eq!(recs[0]["router_id"], "10.0.0.1");
    assert_eq!(recs[0]["local_as"], "65000");
    assert_eq!(recs[0]["bgp_neighbor"], "10.0.0.2");
    assert_eq!(recs[0]["neighbor_as"], "65001");
}

#[test]
fn cisco_nxos_show_cdp_neighbors_parses_multiple_records() {
    let output = include_str!("fixtures/cisco_nxos/show_cdp_neighbors.txt");
    let json_str = netcli_core::parse_json("cisco_nxos", "show_cdp_neighbors", output);
    let v = parse_envelope(&json_str);

    assert_success(&v);
    let recs = records(&v);
    assert_eq!(recs.len(), 2, "should parse two CDP neighbors");
    assert_eq!(recs[0]["neighbor_name"], "switch2.example.com");
    assert_eq!(recs[1]["neighbor_name"], "switch3.example.com");
}

// --- error condition tests ---

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

#[test]
fn command_api_with_hyphenated_command() {
    let output = include_str!("fixtures/arista_eos/show_version.txt");
    let json_str = netcli_core::parse_command_json("arista_eos", "show version", output);
    let v = parse_envelope(&json_str);

    assert_success(&v);
    assert_eq!(v["commandKey"], "show_version");
}

#[test]
fn command_api_unknown_command_returns_template_not_found() {
    let json_str = netcli_core::parse_command_json("cisco_ios", "show magic unicorn", "output");
    let v = parse_envelope(&json_str);

    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "TEMPLATE_NOT_FOUND");
}

// --- parse_records (non-JSON) API tests ---

#[test]
fn parse_records_returns_ok_vec() {
    let output = include_str!("fixtures/cisco_ios/show_version.txt");
    let result = netcli_core::parse_records("cisco_ios", "show_version", output);

    assert!(result.is_ok());
    let recs = result.unwrap();
    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0].get("hostname").unwrap(), "Router01");
}

#[test]
fn parse_records_returns_error_for_unknown_platform() {
    let result = netcli_core::parse_records("nonexistent_os", "show_version", "some output");
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.code(), "TEMPLATE_NOT_FOUND");
    assert!(err.to_string().contains("nonexistent_os"));
}

#[test]
fn parse_records_returns_error_for_empty_platform() {
    let result = netcli_core::parse_records("", "show_version", "some output");
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.code(), "INVALID_INPUT");
    assert!(err.to_string().contains("platform"));
}

#[test]
fn parse_records_returns_error_for_empty_command_key() {
    let result = netcli_core::parse_records("cisco_ios", "", "some output");
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.code(), "INVALID_INPUT");
    assert!(err.to_string().contains("command_key"));
}

#[test]
fn parse_records_returns_error_for_empty_output() {
    let result = netcli_core::parse_records("cisco_ios", "show_version", "");
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert_eq!(err.code(), "INVALID_INPUT");
    assert!(err.to_string().contains("output_text"));
}

#[test]
fn parse_records_multi_record() {
    let output = include_str!("fixtures/cisco_ios/show_interfaces.txt");
    let recs = netcli_core::parse_records("cisco_ios", "show_interfaces", output).unwrap();

    assert_eq!(recs.len(), 2);
    assert_eq!(recs[0].get("interface").unwrap(), "GigabitEthernet0/1");
    assert_eq!(recs[1].get("interface").unwrap(), "GigabitEthernet0/2");
}

// --- parse_command_records (non-JSON) API tests ---

#[test]
fn parse_command_records_returns_ok_vec() {
    let output = include_str!("fixtures/cisco_ios/show_version.txt");
    let result = netcli_core::parse_command_records("cisco_ios", "show version", output);

    assert!(result.is_ok());
    let recs = result.unwrap();
    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0].get("hostname").unwrap(), "Router01");
}

#[test]
fn parse_command_records_normalizes_input() {
    let output = include_str!("fixtures/cisco_ios/show_version.txt");
    let recs = netcli_core::parse_command_records("cisco_ios", "  Show  Version  ", output).unwrap();

    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0].get("hostname").unwrap(), "Router01");
}

#[test]
fn parse_command_records_empty_command_error() {
    let result = netcli_core::parse_command_records("cisco_ios", "", "output");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), "INVALID_INPUT");
}

// --- edge case tests ---

#[test]
fn unrecognized_only_input_returns_empty_records() {
    let garbage = "this is random text\nthat matches nothing\nin the template\n";
    let json_str = netcli_core::parse_json("cisco_ios", "show_version", garbage);
    let v = parse_envelope(&json_str);

    assert_eq!(v["ok"], true, "unrecognized lines should not cause an error");
    let recs = records(&v);
    assert!(recs.is_empty(), "should return empty records for unrecognized input");
}

#[test]
fn output_with_blank_lines_only() {
    let blanks = "\n\n\n\n\n";
    let json_str = netcli_core::parse_json("cisco_ios", "show_version", blanks);
    let v = parse_envelope(&json_str);

    assert_eq!(v["ok"], true);
    assert!(records(&v).is_empty());
}

#[test]
fn json_envelope_success_shape() {
    let output = include_str!("fixtures/cisco_ios/show_version.txt");
    let json_str = netcli_core::parse_json("cisco_ios", "show_version", output);
    let v = parse_envelope(&json_str);

    assert!(v["ok"].is_boolean());
    assert!(v["platform"].is_string());
    assert!(v["commandKey"].is_string());
    assert!(v["records"].is_array());
    assert!(v.get("error").is_none() || v["error"].is_null());
}

#[test]
fn json_envelope_error_shape() {
    let json_str = netcli_core::parse_json("bad_os", "show_version", "text");
    let v = parse_envelope(&json_str);

    assert_eq!(v["ok"], false);
    assert!(v["error"].is_object());
    assert!(v["error"]["code"].is_string());
    assert!(v["error"]["message"].is_string());
    assert!(v.get("records").is_none() || v["records"].is_null());
}

#[test]
fn error_message_includes_context() {
    let result = netcli_core::parse_records("my_platform", "my_command", "text");
    let err = result.unwrap_err();

    let msg = err.to_string();
    assert!(msg.contains("my_platform"), "error message should include platform: {msg}");
    assert!(msg.contains("my_command"), "error message should include command key: {msg}");
}

#[test]
fn parse_error_code_mapping() {
    use netcli_core::ParseError;

    assert_eq!(ParseError::InvalidInput("x").code(), "INVALID_INPUT");
    assert_eq!(
        ParseError::TemplateNotFound { platform: "a".into(), command_key: "b".into() }.code(),
        "TEMPLATE_NOT_FOUND"
    );
    assert_eq!(ParseError::TemplateInvalid("x".into()).code(), "TEMPLATE_INVALID");
    assert_eq!(ParseError::EngineError("x".into()).code(), "PARSE_ERROR");
}

#[test]
fn parse_error_display_formatting() {
    use netcli_core::ParseError;

    let e1 = ParseError::InvalidInput("platform");
    assert_eq!(e1.to_string(), "required input is empty: platform");

    let e2 = ParseError::TemplateNotFound {
        platform: "cisco_ios".into(),
        command_key: "show_version".into(),
    };
    assert_eq!(e2.to_string(), "no template for (cisco_ios, show_version)");

    let e3 = ParseError::TemplateInvalid("bad regex".into());
    assert_eq!(e3.to_string(), "template compilation failed: bad regex");

    let e4 = ParseError::EngineError("something broke".into());
    assert_eq!(e4.to_string(), "parse error: something broke");
}

#[test]
fn parse_error_implements_std_error() {
    fn assert_std_error<T: std::error::Error>() {}
    assert_std_error::<netcli_core::ParseError>();
}


// --- cross-platform prompt tolerance ---

#[test]
fn arista_eos_show_version_tolerates_prompt() {
    let clean = include_str!("fixtures/arista_eos/show_version.txt");
    let with_prompt = format!("switch1#show version\n{clean}\nswitch1#");

    let json_str = netcli_core::parse_json("arista_eos", "show_version", &with_prompt);
    let v = parse_envelope(&json_str);

    assert_success(&v);
    let recs = records(&v);
    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0]["model"], "DCS-7050TX-64");
}

#[test]
fn cisco_nxos_show_version_tolerates_prompt() {
    let clean = include_str!("fixtures/cisco_nxos/show_version.txt");
    let with_prompt = format!("nxos-switch1# show version\n{clean}\nnxos-switch1#");

    let json_str = netcli_core::parse_json("cisco_nxos", "show_version", &with_prompt);
    let v = parse_envelope(&json_str);

    assert_success(&v);
    assert!(!records(&v).is_empty());
}

// --- consistency tests ---

#[test]
fn parse_json_and_parse_records_agree() {
    let output = include_str!("fixtures/cisco_ios/show_version.txt");

    let json_str = netcli_core::parse_json("cisco_ios", "show_version", output);
    let v = parse_envelope(&json_str);
    let json_records = records(&v);

    let direct_records = netcli_core::parse_records("cisco_ios", "show_version", output).unwrap();

    assert_eq!(json_records.len(), direct_records.len());
    for (jr, dr) in json_records.iter().zip(direct_records.iter()) {
        for (key, value) in dr {
            assert_eq!(jr[key], *value, "mismatch for key {key}");
        }
    }
}

#[test]
fn command_and_key_apis_produce_identical_records_for_multiple_platforms() {
    let platforms_and_commands: Vec<(&str, &str, &str)> = vec![
        ("cisco_ios", "show_version", "show version"),
        ("arista_eos", "show_version", "show version"),
        ("cisco_nxos", "show_version", "show version"),
    ];

    for (platform, key, cmd) in platforms_and_commands {
        let fixture = match (platform, key) {
            ("cisco_ios", "show_version") => include_str!("fixtures/cisco_ios/show_version.txt"),
            ("arista_eos", "show_version") => include_str!("fixtures/arista_eos/show_version.txt"),
            ("cisco_nxos", "show_version") => include_str!("fixtures/cisco_nxos/show_version.txt"),
            _ => unreachable!(),
        };

        let by_key = netcli_core::parse_records(platform, key, fixture).unwrap();
        let by_cmd = netcli_core::parse_command_records(platform, cmd, fixture).unwrap();

        assert_eq!(by_key.len(), by_cmd.len(), "record count mismatch for {platform}/{key}");
        assert_eq!(by_key, by_cmd, "records differ for {platform}/{key}");
    }
}

// ========================================================================
// cisco_ios fixture tests
// ========================================================================

#[test]
fn cisco_ios_show_ip_interface_brief() {
    let output = include_str!("fixtures/cisco_ios/show_ip_interface_brief.txt");
    let recs = netcli_core::parse_records("cisco_ios", "show_ip_interface_brief", output).unwrap();

    assert_eq!(recs.len(), 12);
    assert_eq!(recs[0].get("interface").unwrap(), "GigabitEthernet0/0");
    assert_eq!(recs[0].get("ip_address").unwrap(), "10.1.1.1");
    assert_eq!(recs[0].get("status").unwrap(), "up");
    assert_eq!(recs[0].get("proto").unwrap(), "up");
}

// ========================================================================
// cisco_iosxe fixture tests (exercises the cisco_iosxe -> cisco_ios alias)
// ========================================================================

#[test]
fn cisco_iosxe_show_version() {
    let output = include_str!("fixtures/cisco_iosxe/show_version.txt");
    let v = parse_envelope(&netcli_core::parse_json("cisco_iosxe", "show_version", output));

    assert_success(&v);
    let recs = records(&v);
    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0]["hostname"], "my-cisco-device");
    assert_eq!(recs[0]["version"], "17.9.5a");
    assert_eq!(recs[0]["software_image"], "X86_64_LINUX_IOSD-UNIVERSALK9-M");
    assert_eq!(recs[0]["serial"], "[FLM292210DA]");
}

#[test]
fn cisco_iosxe_show_ip_interface_brief() {
    let output = include_str!("fixtures/cisco_iosxe/show_ip_interface_brief.txt");
    let recs = netcli_core::parse_records("cisco_iosxe", "show_ip_interface_brief", output).unwrap();

    assert_eq!(recs.len(), 70);
    assert_eq!(recs[0].get("interface").unwrap(), "GigabitEthernet0/0/0");
    assert_eq!(recs[0].get("ip_address").unwrap(), "10.5.146.254");
    assert_eq!(recs[0].get("status").unwrap(), "up");
}

#[test]
fn cisco_iosxe_show_interfaces_status() {
    let output = include_str!("fixtures/cisco_iosxe/show_interfaces_status.txt");
    let recs = netcli_core::parse_records("cisco_iosxe", "show_interfaces_status", output).unwrap();

    assert_eq!(recs.len(), 10);
    assert_eq!(recs[0].get("port").unwrap(), "Gi0/0/0");
    assert_eq!(recs[0].get("status").unwrap(), "connected");
    assert_eq!(recs[0].get("name").unwrap(), "Uplink-to-Core");
    assert_eq!(recs[0].get("speed").unwrap(), "a-1000");
}

#[test]
fn cisco_iosxe_show_interfaces_transceiver() {
    let output = include_str!("fixtures/cisco_iosxe/show_interfaces_transceiver.txt");
    let recs = netcli_core::parse_records("cisco_iosxe", "show_interfaces_transceiver", output).unwrap();

    assert_eq!(recs.len(), 8);
    assert_eq!(recs[0].get("port").unwrap(), "Te0/0/0");
    assert_eq!(recs[0].get("xcvr_type").unwrap(), "SFP-10G");
    assert_eq!(recs[0].get("vendor").unwrap(), "FINISAR");
    assert_eq!(recs[0].get("tx_power").unwrap(), "-2.1");
    assert_eq!(recs[0].get("rx_power").unwrap(), "-3.4");
}

#[test]
fn cisco_iosxe_show_inventory() {
    let output = include_str!("fixtures/cisco_iosxe/show_inventory.txt");
    let recs = netcli_core::parse_records("cisco_iosxe", "show_inventory", output).unwrap();

    assert_eq!(recs.len(), 11);
    assert_eq!(recs[0].get("name").unwrap(), "Chassis");
    assert_eq!(recs[0].get("pid").unwrap(), "C8300-2N2S-4T2X");
    assert_eq!(recs[0].get("sn").unwrap(), "FLM292210DA");
}

#[test]
fn cisco_iosxe_show_ip_bgp_summary() {
    let output = include_str!("fixtures/cisco_iosxe/show_ip_bgp_summary.txt");
    let recs = netcli_core::parse_records("cisco_iosxe", "show_ip_bgp_summary", output).unwrap();

    assert_eq!(recs.len(), 4);
    assert_eq!(recs[0].get("router_id").unwrap(), "10.0.0.1");
    assert_eq!(recs[0].get("bgp_neighbor").unwrap(), "10.0.0.2");
    assert_eq!(recs[0].get("neighbor_as").unwrap(), "65001");
    assert_eq!(recs[0].get("state_or_prefixes_received").unwrap(), "625");
}

#[test]
fn cisco_iosxe_show_ip_route_summary() {
    let output = include_str!("fixtures/cisco_iosxe/show_ip_route_summary.txt");
    let recs = netcli_core::parse_records("cisco_iosxe", "show_ip_route_summary", output).unwrap();

    assert_eq!(recs.len(), 11);
    assert_eq!(recs[0].get("route_source").unwrap(), "application");
    assert_eq!(recs[1].get("route_source").unwrap(), "connected");
    assert_eq!(recs[1].get("subnets").unwrap(), "12");
}

#[test]
fn cisco_iosxe_show_platform_format_mismatch() {
    let output = include_str!("fixtures/cisco_iosxe/show_platform.txt");
    let recs = netcli_core::parse_records("cisco_iosxe", "show_platform", output).unwrap();

    // The existing show_platform template targets the IOS switch-stack format
    // (Switch/Ports/Model). IOS-XE uses a different Slot/Type/State layout,
    // so the template matches zero lines.
    assert_eq!(recs.len(), 0);
}

#[test]
fn cisco_iosxe_show_environment_sensor_rpm() {
    let output = include_str!("fixtures/cisco_iosxe/show_environment_sensor_rpm.txt");
    let recs = netcli_core::parse_records("cisco_iosxe", "show_environment_sensor_rpm", output).unwrap();

    assert_eq!(recs.len(), 5);
    assert_eq!(recs[0].get("sensor_name").unwrap(), "fan0");
    assert_eq!(recs[0].get("location").unwrap(), "P0");
    assert_eq!(recs[0].get("state").unwrap(), "Normal");
    assert_eq!(recs[0].get("reading").unwrap(), "9936");
}

#[test]
fn cisco_iosxe_show_environment_sensor_temperature() {
    let output = include_str!("fixtures/cisco_iosxe/show_environment_sensor_temperature.txt");
    let recs = netcli_core::parse_records("cisco_iosxe", "show_environment_sensor_temperature", output).unwrap();

    assert_eq!(recs.len(), 8);
    assert_eq!(recs[0].get("sensor_name").unwrap(), "Temp 1");
    assert_eq!(recs[0].get("location").unwrap(), "P0");
    assert_eq!(recs[0].get("reading").unwrap(), "25");
}

#[test]
fn cisco_iosxe_show_interfaces_transceiver_detail() {
    let output = include_str!("fixtures/cisco_iosxe/show_interfaces_transceiver_detail.txt");
    let recs = netcli_core::parse_records("cisco_iosxe", "show_interfaces_transceiver_detail", output).unwrap();

    assert_eq!(recs.len(), 8);
    assert_eq!(recs[0].get("port").unwrap(), "Te0/0/0");
    assert_eq!(recs[0].get("temperature").unwrap(), "31.7");
    assert_eq!(recs[0].get("voltage").unwrap(), "3.29");
    assert_eq!(recs[0].get("tx_power").unwrap(), "-2.1");
    assert_eq!(recs[0].get("rx_power").unwrap(), "-3.4");
}

#[test]
fn cisco_iosxe_show_platform_resources() {
    let output = include_str!("fixtures/cisco_iosxe/show_platform_resources.txt");
    let recs = netcli_core::parse_records("cisco_iosxe", "show_platform_resources", output).unwrap();

    assert_eq!(recs.len(), 18);
    assert_eq!(recs[0].get("processor").unwrap(), "RP0");
    assert_eq!(recs[0].get("resource").unwrap(), "Control Processor");
    assert_eq!(recs[0].get("usage").unwrap(), "6.60%");
    assert_eq!(recs[0].get("state").unwrap(), "H");
}

#[test]
fn cisco_iosxe_show_interface_ge_include_bia() {
    let output = include_str!("fixtures/cisco_iosxe/show_interface_ge_include_bia.txt");
    let recs = netcli_core::parse_records("cisco_iosxe", "show_interface_ge_include_bia", output).unwrap();

    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0].get("hardware_type").unwrap(), "4x1G-2xSFP+");
    assert_eq!(recs[0].get("mac_address").unwrap(), "1096.c62a.8db0");
    assert_eq!(recs[0].get("bia").unwrap(), "1096.c62a.8db0");
}

// ========================================================================
// Alias identity: cisco_ios and cisco_iosxe produce identical records
// ========================================================================

#[test]
fn cisco_ios_and_iosxe_alias_produce_identical_records() {
    let fixtures: Vec<(&str, &str)> = vec![
        ("show_version", include_str!("fixtures/cisco_ios/show_version.txt")),
        ("show_ip_interface_brief", include_str!("fixtures/cisco_ios/show_ip_interface_brief.txt")),
        ("show_ip_bgp_summary", include_str!("fixtures/cisco_ios/show_ip_bgp_summary.txt")),
    ];

    for (key, output) in fixtures {
        let ios = netcli_core::parse_records("cisco_ios", key, output).unwrap();
        let iosxe = netcli_core::parse_records("cisco_iosxe", key, output).unwrap();

        assert_eq!(ios, iosxe, "cisco_ios and cisco_iosxe should produce identical records for {key}");
    }
}

// ========================================================================
// cisco_iosxe command API tests (exercises alias + normalization)
// ========================================================================

#[test]
fn cisco_iosxe_command_api_show_version() {
    let output = include_str!("fixtures/cisco_iosxe/show_version.txt");
    let v = parse_envelope(&netcli_core::parse_command_json("cisco_iosxe", "show version", output));

    assert_success(&v);
    assert_eq!(v["commandKey"], "show_version");
    assert_eq!(records(&v)[0]["hostname"], "my-cisco-device");
}

#[test]
fn cisco_iosxe_command_api_show_interfaces_status() {
    let output = include_str!("fixtures/cisco_iosxe/show_interfaces_status.txt");
    let recs = netcli_core::parse_command_records("cisco_iosxe", "show interfaces status", output).unwrap();

    assert_eq!(recs.len(), 10);
    assert_eq!(recs[0].get("port").unwrap(), "Gi0/0/0");
}

// ========================================================================
// cisco_iosxe show_line tests (new template)
// ========================================================================

#[test]
fn cisco_iosxe_show_line() {
    let output = include_str!("fixtures/cisco_iosxe/show_line.txt");
    let recs = netcli_core::parse_records("cisco_iosxe", "show_line", output).unwrap();

    assert_eq!(recs.len(), 5);
    assert_eq!(recs[0].get("tty").unwrap(), "0");
    assert_eq!(recs[0].get("line").unwrap(), "0");
    assert_eq!(recs[0].get("type").unwrap(), "CTY");
    assert_eq!(recs[0].get("noise").unwrap(), "1");

    assert_eq!(recs[1].get("tty").unwrap(), "1");
    assert_eq!(recs[1].get("type").unwrap(), "AUX");
    assert_eq!(recs[1].get("tx_rx").unwrap(), "9600/9600");

    assert_eq!(recs[2].get("tty").unwrap(), "1/0/0");
    assert_eq!(recs[2].get("line").unwrap(), "98");
    assert_eq!(recs[2].get("type").unwrap(), "TTY");
    assert_eq!(recs[2].get("uses").unwrap(), "106");
}

// ========================================================================
// Abbreviated command variants (separate commandKeys, same template)
// ========================================================================

#[test]
fn cisco_iosxe_show_ip_int_br() {
    let output = include_str!("fixtures/cisco_iosxe/show_ip_int_br.txt");
    let recs = netcli_core::parse_records("cisco_iosxe", "show_ip_int_br", output).unwrap();

    assert_eq!(recs.len(), 3);
    assert_eq!(recs[0].get("interface").unwrap(), "GigabitEthernet0/0/0");
    assert_eq!(recs[0].get("ip_address").unwrap(), "10.5.146.254");
    assert_eq!(recs[0].get("status").unwrap(), "up");
    assert_eq!(recs[1].get("interface").unwrap(), "GigabitEthernet0/0/1");
    assert_eq!(recs[1].get("status").unwrap(), "administratively down");
}

#[test]
fn cisco_iosxe_show_ip_int_br_ex_unas() {
    let output = include_str!("fixtures/cisco_iosxe/show_ip_int_br_ex_unas.txt");
    let recs = netcli_core::parse_records("cisco_iosxe", "show_ip_int_br_ex_unas", output).unwrap();

    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0].get("interface").unwrap(), "GigabitEthernet0/0/0");
    assert_eq!(recs[0].get("ip_address").unwrap(), "10.5.146.254");
    assert_eq!(recs[0].get("status").unwrap(), "up");
    assert_eq!(recs[0].get("proto").unwrap(), "up");
}

// ========================================================================
// DriveNets DNOS fixture tests
// ========================================================================

#[test]
fn dnos_show_system_version() {
    let output = include_str!("fixtures/drivenets_dnos/show_system_version.txt");
    let recs = netcli_core::parse_records("drivenets_dnos", "show_system_version", output).unwrap();

    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0].get("system_name").unwrap(), "DN-SA-01");
    assert_eq!(recs[0].get("hardware_model").unwrap(), "NCF-400G");
    assert_eq!(recs[0].get("software_version").unwrap(), "DNOS 25.4.0");
    assert_eq!(recs[0].get("serial_number").unwrap(), "DN2024010001");
    assert_eq!(recs[0].get("system_uptime").unwrap(), "127 days, 14 hours, 33 minutes");
    assert_eq!(recs[0].get("last_reboot").unwrap(), "2024-09-05 10:22:15 UTC");
}

#[test]
fn dnos_show_system_status() {
    let output = include_str!("fixtures/drivenets_dnos/show_system_status.txt");
    let recs = netcli_core::parse_records("drivenets_dnos", "show_system_status", output).unwrap();

    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0].get("system_name").unwrap(), "DN-SA-01");
    assert_eq!(recs[0].get("software_version").unwrap(), "DNOS 21.3.5");
    assert_eq!(recs[0].get("overall_status").unwrap(), "Healthy");
    assert_eq!(recs[0].get("active_ncps").unwrap(), "4");
    assert_eq!(recs[0].get("cpu_control").unwrap(), "45%");
    assert_eq!(recs[0].get("memory_control").unwrap(), "62%");
}

#[test]
fn dnos_show_interfaces_brief() {
    let output = include_str!("fixtures/drivenets_dnos/show_interfaces_brief.txt");
    let recs = netcli_core::parse_records("drivenets_dnos", "show_interfaces_brief", output).unwrap();

    assert_eq!(recs.len(), 28);
    assert_eq!(recs[0].get("interface").unwrap(), "bundle-12");
    assert_eq!(recs[0].get("admin").unwrap(), "enabled");
    assert_eq!(recs[0].get("link").unwrap(), "up");
    assert_eq!(recs[0].get("speed").unwrap(), "200Gbps");
    assert_eq!(recs[0].get("ipv4_address").unwrap(), "192.168.12.0/31");
}

#[test]
fn dnos_show_interfaces_detail() {
    let output = include_str!("fixtures/drivenets_dnos/show_interfaces_detail.txt");
    let recs = netcli_core::parse_records("drivenets_dnos", "show_interfaces_detail", output).unwrap();

    assert_eq!(recs.len(), 52);
    assert_eq!(recs[0].get("interface").unwrap(), "bundle-12");
    assert_eq!(recs[0].get("admin_state").unwrap(), "enabled");
    assert_eq!(recs[0].get("mac_address").unwrap(), "84:40:76:d9:0e:4e");
    assert_eq!(recs[0].get("speed").unwrap(), "200Gbps");
    assert_eq!(recs[0].get("ipv4_address").unwrap(), "192.168.12.0/31");
    assert_eq!(recs[0].get("l2_mtu").unwrap(), "1514");
}

#[test]
fn dnos_show_interfaces_transceiver() {
    let output = include_str!("fixtures/drivenets_dnos/show_interfaces_transceiver.txt");
    let recs = netcli_core::parse_records("drivenets_dnos", "show_interfaces_transceiver", output).unwrap();

    assert_eq!(recs.len(), 40);
    assert_eq!(recs[0].get("interface").unwrap(), "ge100-0/0/0");
    assert_eq!(recs[0].get("identifier").unwrap(), "QSFP28");
    assert_eq!(recs[0].get("vendor_name").unwrap(), "FINISAR CORP.");
    assert_eq!(recs[0].get("vendor_sn").unwrap(), "U4DADNU");
}

#[test]
fn dnos_show_config_flatten() {
    let output = include_str!("fixtures/drivenets_dnos/show_config_flatten.txt");
    let recs = netcli_core::parse_records("drivenets_dnos", "show_config_flatten", output).unwrap();

    assert_eq!(recs.len(), 36);
    assert_eq!(recs[0].get("line").unwrap(), "hostname DN-SA-01");
    assert_eq!(recs[1].get("line").unwrap(), "interface Management0");
}

#[test]
fn dnos_show_lldp() {
    let output = include_str!("fixtures/drivenets_dnos/show_lldp.txt");
    let recs = netcli_core::parse_records("drivenets_dnos", "show_lldp", output).unwrap();

    assert_eq!(recs.len(), 6);
    assert_eq!(recs[0].get("interface").unwrap(), "ge100-0/0/0");
    assert_eq!(recs[0].get("transmit").unwrap(), "enabled");
    assert_eq!(recs[0].get("receive").unwrap(), "enabled");
    assert_eq!(recs[0].get("keepalive").unwrap(), "30");
    assert_eq!(recs[0].get("holdtime").unwrap(), "120");
}

#[test]
fn dnos_show_lldp_counters() {
    let output = include_str!("fixtures/drivenets_dnos/show_lldp_counters.txt");
    let recs = netcli_core::parse_records("drivenets_dnos", "show_lldp_counters", output).unwrap();

    assert_eq!(recs.len(), 6);
    assert_eq!(recs[0].get("interface").unwrap(), "ge100-0/0/0");
    assert_eq!(recs[0].get("pdu_rx").unwrap(), "49278");
    assert_eq!(recs[0].get("pdu_tx").unwrap(), "49278");
    assert_eq!(recs[0].get("inserted").unwrap(), "1");
}

#[test]
fn dnos_show_lldp_neighbors() {
    let output = include_str!("fixtures/drivenets_dnos/show_lldp_neighbors.txt");
    let recs = netcli_core::parse_records("drivenets_dnos", "show_lldp_neighbors", output).unwrap();

    assert_eq!(recs.len(), 6);
    assert_eq!(recs[0].get("interface").unwrap(), "ge100-0/0/0");
    assert_eq!(recs[0].get("neighbor_name").unwrap(), "DN-SA-02");
    assert_eq!(recs[0].get("neighbor_interface").unwrap(), "ge100-0/0/0");
    assert_eq!(recs[0].get("neighbor_ttl").unwrap(), "120");
}

#[test]
fn dnos_show_route_summary() {
    let output = include_str!("fixtures/drivenets_dnos/show_route_summary.txt");
    let recs = netcli_core::parse_records("drivenets_dnos", "show_route_summary", output).unwrap();

    assert_eq!(recs.len(), 6);
    assert_eq!(recs[0].get("route_source").unwrap(), "connected");
    assert_eq!(recs[0].get("routes").unwrap(), "12");
    assert_eq!(recs[2].get("route_source").unwrap(), "bgp");
    assert_eq!(recs[2].get("paths").unwrap(), "2500");
    assert_eq!(recs[5].get("route_source").unwrap(), "Total");
}

#[test]
fn dnos_show_system_hardware() {
    let output = include_str!("fixtures/drivenets_dnos/show_system_hardware.txt");
    let recs = netcli_core::parse_records("drivenets_dnos", "show_system_hardware", output).unwrap();

    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0].get("model").unwrap(), "NCP-40C");
    assert_eq!(recs[0].get("serial_number").unwrap(), "WDY1957500030");
    assert_eq!(recs[0].get("chassis_mac").unwrap(), "e8:c5:7a:03:56:1a");
    assert_eq!(recs[0].get("host_name").unwrap(), "WDY1957500030");
}

#[test]
fn dnos_show_system_hardware_cpu() {
    let output = include_str!("fixtures/drivenets_dnos/show_system_hardware_cpu.txt");
    let recs = netcli_core::parse_records("drivenets_dnos", "show_system_hardware_cpu", output).unwrap();

    assert_eq!(recs.len(), 16);
    assert_eq!(recs[0].get("cpu").unwrap(), "0");
    assert_eq!(recs[0].get("use_percent").unwrap(), "28");
    assert_eq!(recs[15].get("cpu").unwrap(), "15");
    assert_eq!(recs[15].get("use_percent").unwrap(), "45");
}

#[test]
fn dnos_show_system_hardware_fan() {
    let output = include_str!("fixtures/drivenets_dnos/show_system_hardware_fan.txt");
    let recs = netcli_core::parse_records("drivenets_dnos", "show_system_hardware_fan", output).unwrap();

    assert_eq!(recs.len(), 8);
    assert_eq!(recs[0].get("fan_id").unwrap(), "FAN0_RPM");
    assert_eq!(recs[0].get("status").unwrap(), "OK");
    assert_eq!(recs[0].get("speed_rpm").unwrap(), "6300");
    assert_eq!(recs[0].get("max_rpm").unwrap(), "28500");
}

#[test]
fn dnos_show_system_hardware_inventory() {
    let output = include_str!("fixtures/drivenets_dnos/show_system_hardware_inventory.txt");
    let recs = netcli_core::parse_records("drivenets_dnos", "show_system_hardware_inventory", output).unwrap();

    assert_eq!(recs.len(), 2);
    assert_eq!(recs[0].get("component").unwrap(), "dn-ncc-0");
    assert_eq!(recs[0].get("model").unwrap(), "NCP-40C");
    assert_eq!(recs[1].get("component").unwrap(), "dn-ncp-0");
    assert_eq!(recs[1].get("serial_number").unwrap(), "WDY1957500030");
}

#[test]
fn dnos_show_system_hardware_power() {
    let output = include_str!("fixtures/drivenets_dnos/show_system_hardware_power.txt");
    let recs = netcli_core::parse_records("drivenets_dnos", "show_system_hardware_power", output).unwrap();

    assert_eq!(recs.len(), 2);
    assert_eq!(recs[0].get("psu_id").unwrap(), "0");
    assert_eq!(recs[0].get("status").unwrap(), "OK");
    assert_eq!(recs[0].get("type").unwrap(), "AC 110V-220V");
    assert_eq!(recs[0].get("serial").unwrap(), "S0A030Z851915000883");
}

#[test]
fn dnos_show_system_hardware_temperature() {
    let output = include_str!("fixtures/drivenets_dnos/show_system_hardware_temperature.txt");
    let recs = netcli_core::parse_records("drivenets_dnos", "show_system_hardware_temperature", output).unwrap();

    assert_eq!(recs.len(), 20);
    assert_eq!(recs[0].get("sensor_name").unwrap(), "PSU0_TEMP");
    assert_eq!(recs[0].get("temperature").unwrap(), "33.0");
    assert_eq!(recs[0].get("status").unwrap(), "OK");
    assert_eq!(recs[0].get("high_warning").unwrap(), "65");
    assert_eq!(recs[0].get("high_critical").unwrap(), "70");
}

// ========================================================================
// DNOS command API test
// ========================================================================

#[test]
fn dnos_command_api_show_system_version() {
    let output = include_str!("fixtures/drivenets_dnos/show_system_version.txt");
    let v = parse_envelope(&netcli_core::parse_command_json("drivenets_dnos", "show system version", output));

    assert_success(&v);
    assert_eq!(v["commandKey"], "show_system_version");
    assert_eq!(records(&v)[0]["system_name"], "DN-SA-01");
}

// ========================================================================
// Phase 1 tests: normalize_command fixes (hyphens, pipes, abbreviations)
// ========================================================================

#[test]
fn command_api_hyphenated_running_config_interface() {
    let output = include_str!("fixtures/cisco_ios/show_version.txt");
    let v = parse_envelope(&netcli_core::parse_json(
        "cisco_ios",
        "show_running_config_interface",
        output,
    ));
    assert_eq!(v["ok"], true, "show_running_config_interface should be found in registry");
}

#[test]
fn command_api_pipe_config_flatten_dnos() {
    let output = include_str!("fixtures/drivenets_dnos/show_config_flatten.txt");
    let v = parse_envelope(&netcli_core::parse_command_json(
        "drivenets_dnos",
        "show config | flatten",
        output,
    ));
    assert_success(&v);
    assert_eq!(v["commandKey"], "show_config_flatten");
}

#[test]
fn command_api_abbreviated_show_version() {
    let output = include_str!("fixtures/cisco_ios/show_version.txt");
    let v = parse_envelope(&netcli_core::parse_command_json("cisco_ios", "sho version", output));
    assert_success(&v);

    let v2 = parse_envelope(&netcli_core::parse_command_json("cisco_ios", "sh ver", output));
    assert_success(&v2);
}

#[test]
fn command_api_abbreviated_show_ip_int_brief() {
    let output = include_str!("fixtures/cisco_ios/show_ip_interface_brief.txt");
    let v = parse_envelope(&netcli_core::parse_command_json(
        "cisco_ios",
        "sho ip int br",
        output,
    ));
    assert_success(&v);
    assert!(
        !records(&v).is_empty(),
        "abbreviated 'sho ip int br' should parse via expansion"
    );
}

#[test]
fn command_api_mac_address_table_with_hyphens() {
    let output = "some dummy text\n";
    let v = parse_envelope(&netcli_core::parse_command_json(
        "cisco_ios",
        "show mac address-table",
        output,
    ));
    assert_eq!(v["ok"], true, "show_mac_address_table should resolve via hyphen normalization");
}

#[test]
fn command_api_spanning_tree_with_hyphens() {
    let output = "some dummy text\n";
    let v = parse_envelope(&netcli_core::parse_command_json(
        "cisco_ios",
        "show spanning-tree",
        output,
    ));
    assert_eq!(v["ok"], true, "show_spanning_tree should resolve via hyphen normalization");
}

// ========================================================================
// Phase 2 tests: platform aliases (nokia_sros, cisco_iosxr)
// ========================================================================

#[test]
fn nokia_sros_alias_resolves_to_alcatel() {
    let result = netcli_core::parse_records("nokia_sros", "show_system_cpu", "dummy\n");
    assert!(result.is_ok() || {
        let e = result.unwrap_err();
        e.code() != "TEMPLATE_NOT_FOUND"
    }, "nokia_sros should alias to alcatel_sros and find show_system_cpu");
}

#[test]
fn nokia_sros_alias_lookup_works() {
    let v = parse_envelope(&netcli_core::parse_json("nokia_sros", "show_port", "dummy\n"));
    assert_eq!(v["ok"], true, "nokia_sros should alias to alcatel_sros");
}

#[test]
fn cisco_iosxr_alias_resolves_to_xr() {
    let v = parse_envelope(&netcli_core::parse_json("cisco_iosxr", "show_version", "dummy\n"));
    assert_eq!(v["ok"], true, "cisco_iosxr should alias to cisco_xr");
}

#[test]
fn cisco_iosxr_alias_show_ip_bgp_summary() {
    let v = parse_envelope(&netcli_core::parse_json(
        "cisco_iosxr",
        "show_ip_bgp_summary",
        "dummy\n",
    ));
    assert_eq!(v["ok"], true, "cisco_iosxr should resolve show_ip_bgp_summary via cisco_xr");
}

// ========================================================================
// Phase 3 tests: registry aliases (singular/plural, expanded abbreviations)
// ========================================================================

#[test]
fn cisco_ios_show_interface_singular_alias() {
    let output = include_str!("fixtures/cisco_ios/show_interfaces.txt");
    let v = parse_envelope(&netcli_core::parse_json("cisco_ios", "show_interface", output));
    assert_success(&v);
    assert!(!records(&v).is_empty(), "show_interface (singular) should alias to show_interfaces");
}

#[test]
fn dnos_show_interface_brief_singular_alias() {
    let output = include_str!("fixtures/drivenets_dnos/show_interfaces_brief.txt");
    let v = parse_envelope(&netcli_core::parse_json(
        "drivenets_dnos",
        "show_interface_brief",
        output,
    ));
    assert_success(&v);
    assert!(
        !records(&v).is_empty(),
        "show_interface_brief (singular) should alias to show_interfaces_brief"
    );
}

#[test]
fn cisco_ios_expanded_abbreviated_command_entry() {
    let output = include_str!("fixtures/cisco_ios/show_ip_interface_brief.txt");
    let v = parse_envelope(&netcli_core::parse_json(
        "cisco_ios",
        "show_ip_interface_brief_exclude_unassigned",
        output,
    ));
    assert_success(&v);
}

// ========================================================================
// Phase 4-9 tests: new template lookups (verify templates compile)
// ========================================================================

#[test]
fn new_cisco_ios_templates_compile() {
    let keys = [
        "show_environment",
        "show_environment_all",
        "show_memory_statistics",
        "show_ip_protocols",
        "show_diag",
        "show_spanning_tree_summary",
        "show_aaa_sessions",
        "show_privilege",
        "show_crypto_key_mypubkey_rsa",
        "show_ntp_status",
        "show_terminal",
        "show_license_udi",
    ];
    for key in keys {
        let v = parse_envelope(&netcli_core::parse_json("cisco_ios", key, "dummy line\n"));
        assert_eq!(v["ok"], true, "cisco_ios template for {key} should compile and return ok");
    }
}

#[test]
fn new_cisco_xr_templates_compile() {
    let keys = [
        "show_controllers_optics",
        "show_interface_accounting",
        "show_controllers_fec",
        "show_running_config_interface",
    ];
    for key in keys {
        let v = parse_envelope(&netcli_core::parse_json("cisco_xr", key, "dummy line\n"));
        assert_eq!(v["ok"], true, "cisco_xr template for {key} should compile and return ok");
    }
}

#[test]
fn new_cisco_xr_via_iosxr_alias_templates_compile() {
    let keys = [
        "show_controllers_optics",
        "show_interface_accounting",
        "show_controllers_fec",
        "show_running_config_interface",
    ];
    for key in keys {
        let v = parse_envelope(&netcli_core::parse_json("cisco_iosxr", key, "dummy line\n"));
        assert_eq!(
            v["ok"], true,
            "cisco_iosxr (alias) template for {key} should compile and return ok"
        );
    }
}

#[test]
fn new_dnos_templates_compile() {
    let keys = [
        "show_version",
        "show_system_hardware_psu",
        "show_interface",
        "show_interface_fec",
        "show_interface_transceiver",
        "show_interface_counters",
        "show_interface_fec_counters",
        "show_lldp_neighbor_detail",
        "show_bfd_session",
    ];
    for key in keys {
        let v = parse_envelope(&netcli_core::parse_json("drivenets_dnos", key, "dummy line\n"));
        assert_eq!(
            v["ok"], true,
            "drivenets_dnos template for {key} should compile and return ok"
        );
    }
}

#[test]
fn new_nokia_templates_compile() {
    let keys = [
        "show_version",
        "show_system_information",
        "show_system_location",
        "show_chassis",
        "show_chassis_detail",
        "show_bof",
        "show_mda",
        "show_port_optical",
        "show_port_statistics",
        "show_port_ethernet_fec",
        "show_router_bgp_summary",
        "show_system_lldp_neighbor",
        "show_router_bfd_session_detail",
        "show_service",
        "show_snmp_location",
        "admin_display_config",
    ];
    for key in keys {
        let v = parse_envelope(&netcli_core::parse_json("alcatel_sros", key, "dummy line\n"));
        assert_eq!(
            v["ok"], true,
            "alcatel_sros template for {key} should compile and return ok"
        );
    }
}

#[test]
fn new_nokia_via_alias_templates_compile() {
    let keys = [
        "show_version",
        "show_system_information",
        "show_chassis",
        "show_router_bgp_summary",
        "show_service",
    ];
    for key in keys {
        let v = parse_envelope(&netcli_core::parse_json("nokia_sros", key, "dummy line\n"));
        assert_eq!(
            v["ok"], true,
            "nokia_sros (alias) template for {key} should compile and return ok"
        );
    }
}

#[test]
fn new_junos_templates_compile() {
    let keys = [
        "show_system_information",
        "show_system_location",
        "show_system_contact_information",
        "show_system_alarms",
        "show_interfaces_terse",
        "show_chassis_environment",
    ];
    for key in keys {
        let v = parse_envelope(&netcli_core::parse_json("juniper_junos", key, "dummy line\n"));
        assert_eq!(
            v["ok"], true,
            "juniper_junos template for {key} should compile and return ok"
        );
    }
}

#[test]
fn new_arista_eos_templates_resolve() {
    let keys = [
        "show_system_environment_all",
        "show_system_environment_cooling",
        "show_system_environment_temperature",
        "show_system_environment_power",
    ];
    for key in keys {
        let v = parse_envelope(&netcli_core::parse_json("arista_eos", key, "dummy line\n"));
        let err_code = v["error"]["code"].as_str().unwrap_or("");
        assert_ne!(
            err_code, "TEMPLATE_NOT_FOUND",
            "arista_eos should have a template for {key}"
        );
        assert_ne!(
            err_code, "TEMPLATE_INVALID",
            "arista_eos template for {key} should compile"
        );
    }
}

// ========================================================================
// Command API integration: full commands resolve after all fixes
// ========================================================================

#[test]
fn command_api_nokia_show_system_cpu() {
    let v = parse_envelope(&netcli_core::parse_command_json(
        "nokia_sros",
        "show system cpu",
        "dummy line\n",
    ));
    assert_eq!(v["ok"], true, "nokia_sros 'show system cpu' should resolve");
}

#[test]
fn command_api_nokia_show_router_interface() {
    let v = parse_envelope(&netcli_core::parse_command_json(
        "nokia_sros",
        "show router interface",
        "dummy line\n",
    ));
    assert_eq!(v["ok"], true, "nokia_sros 'show router interface' should resolve");
}

#[test]
fn command_api_cisco_xr_show_version() {
    let v = parse_envelope(&netcli_core::parse_command_json(
        "cisco_iosxr",
        "show version",
        "dummy line\n",
    ));
    assert_eq!(v["ok"], true, "cisco_iosxr 'show version' should resolve via alias");
}

#[test]
fn command_api_junos_show_chassis_hardware() {
    let v = parse_envelope(&netcli_core::parse_command_json(
        "juniper_junos",
        "show chassis hardware",
        "dummy line\n",
    ));
    assert_eq!(v["ok"], true, "juniper_junos 'show chassis hardware' should resolve");
}

#[test]
fn command_api_arista_show_system_environment_temperature() {
    let v = parse_envelope(&netcli_core::parse_command_json(
        "arista_eos",
        "show system environment temperature",
        "dummy line\n",
    ));
    let err_code = v["error"]["code"].as_str().unwrap_or("");
    assert_ne!(
        err_code, "TEMPLATE_NOT_FOUND",
        "arista_eos 'show system environment temperature' should resolve"
    );
}

#[test]
fn command_api_dnos_show_interfaces_brief() {
    let output = include_str!("fixtures/drivenets_dnos/show_interfaces_brief.txt");
    let v = parse_envelope(&netcli_core::parse_command_json(
        "drivenets_dnos",
        "show interfaces brief",
        output,
    ));
    assert_success(&v);
    assert!(!records(&v).is_empty());
}

#[test]
fn command_api_dnos_show_interface_brief_singular() {
    let output = include_str!("fixtures/drivenets_dnos/show_interfaces_brief.txt");
    let v = parse_envelope(&netcli_core::parse_command_json(
        "drivenets_dnos",
        "show interface brief",
        output,
    ));
    assert_success(&v);
    assert!(
        !records(&v).is_empty(),
        "singular 'show interface brief' should work via alias"
    );
}
