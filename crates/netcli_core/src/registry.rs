use std::collections::HashMap;
use std::sync::OnceLock;

use include_dir::{include_dir, Dir};
use serde::Deserialize;

static RESOURCES: Dir = include_dir!("$CARGO_MANIFEST_DIR/resources");

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub(crate) struct RegistryEntry {
    pub platform: String,
    #[serde(rename = "commandKey")]
    pub command_key: String,
    pub template: String,
    pub shape: String,
}

#[derive(Deserialize)]
struct RegistryFile {
    templates: Vec<RegistryEntry>,
}

type Key = (String, String);

fn registry() -> &'static HashMap<Key, RegistryEntry> {
    static INSTANCE: OnceLock<HashMap<Key, RegistryEntry>> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        let json = RESOURCES
            .get_file("registry.json")
            .and_then(|f| f.contents_utf8())
            .expect("embedded registry.json missing");

        let file: RegistryFile =
            serde_json::from_str(json).expect("registry.json is not valid JSON");

        file.templates
            .into_iter()
            .map(|e| ((e.platform.clone(), e.command_key.clone()), e))
            .collect()
    })
}

fn resolve_platform(platform: &str) -> &str {
    match platform {
        "cisco_iosxe" => "cisco_ios",
        "nokia_sros" => "alcatel_sros",
        "cisco_iosxr" => "cisco_xr",
        other => other,
    }
}

pub(crate) fn lookup(platform: &str, command_key: &str) -> Option<&'static RegistryEntry> {
    let canonical = resolve_platform(platform);
    registry().get(&(canonical.into(), command_key.into()))
}

fn expand_abbreviation(word: &str) -> String {
    match word {
        "sh" | "sho" => "show".into(),
        "int" => "interface".into(),
        "br" => "brief".into(),
        "ex" => "exclude".into(),
        "unas" => "unassigned".into(),
        "desc" => "description".into(),
        "neigh" | "nei" => "neighbors".into(),
        "sum" => "summary".into(),
        "det" => "detail".into(),
        "inv" => "inventory".into(),
        "env" => "environment".into(),
        "ver" => "version".into(),
        "trans" => "transceiver".into(),
        "stat" => "status".into(),
        "proc" => "processes".into(),
        "addr" => "address".into(),
        "conf" => "config".into(),
        "run" => "running".into(),
        "temp" => "temperature".into(),
        other => other.into(),
    }
}

fn normalize_raw(command: &str) -> String {
    command
        .split(|c: char| c.is_whitespace() || c == '|')
        .filter(|s| !s.is_empty())
        .map(|word| {
            let w = word.to_ascii_lowercase().replace('-', "_");
            match w.as_str() {
                "sh" | "sho" => "show".into(),
                _ => w,
            }
        })
        .collect::<Vec<String>>()
        .join("_")
}

pub(crate) fn normalize_command(command: &str) -> String {
    command
        .split(|c: char| c.is_whitespace() || c == '|')
        .filter(|s| !s.is_empty())
        .map(|word| expand_abbreviation(&word.to_ascii_lowercase().replace('-', "_")))
        .collect::<Vec<String>>()
        .join("_")
}

pub(crate) fn lookup_command(
    platform: &str,
    command: &str,
) -> (String, Option<&'static RegistryEntry>) {
    let expanded = normalize_command(command);
    if let Some(entry) = lookup(platform, &expanded) {
        return (expanded, Some(entry));
    }
    let raw = normalize_raw(command);
    if raw != expanded {
        if let Some(entry) = lookup(platform, &raw) {
            return (raw, Some(entry));
        }
    }
    (expanded, None)
}

pub(crate) fn load_template_text(entry: &RegistryEntry) -> Option<&'static str> {
    RESOURCES
        .get_file(&entry.template)
        .and_then(|f| f.contents_utf8())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_simple_command() {
        assert_eq!(normalize_command("show version"), "show_version");
    }

    #[test]
    fn normalize_multi_word() {
        assert_eq!(normalize_command("show ip bgp summary"), "show_ip_bgp_summary");
    }

    #[test]
    fn normalize_trims_whitespace() {
        assert_eq!(normalize_command("  show version  "), "show_version");
    }

    #[test]
    fn normalize_collapses_multiple_spaces() {
        assert_eq!(normalize_command("show    ip    route"), "show_ip_route");
    }

    #[test]
    fn normalize_lowercases() {
        assert_eq!(normalize_command("Show Version"), "show_version");
        assert_eq!(normalize_command("SHOW IP BGP"), "show_ip_bgp");
    }

    #[test]
    fn normalize_handles_tabs_and_mixed_whitespace() {
        assert_eq!(normalize_command("show\t\tversion"), "show_version");
        assert_eq!(normalize_command(" show \t ip \n route "), "show_ip_route");
    }

    #[test]
    fn normalize_empty_string() {
        assert_eq!(normalize_command(""), "");
    }

    #[test]
    fn normalize_whitespace_only() {
        assert_eq!(normalize_command("   "), "");
        assert_eq!(normalize_command("\t\n"), "");
    }

    #[test]
    fn normalize_already_normalized() {
        assert_eq!(normalize_command("show_version"), "show_version");
    }

    #[test]
    fn normalize_converts_hyphens_to_underscores() {
        assert_eq!(normalize_command("show running-config"), "show_running_config");
        assert_eq!(normalize_command("show mac address-table"), "show_mac_address_table");
        assert_eq!(normalize_command("show spanning-tree"), "show_spanning_tree");
    }

    #[test]
    fn normalize_strips_pipe_characters() {
        assert_eq!(normalize_command("show config | flatten"), "show_config_flatten");
        assert_eq!(
            normalize_command("show running-config | include hostname"),
            "show_running_config_include_hostname"
        );
    }

    #[test]
    fn normalize_expands_abbreviations() {
        assert_eq!(normalize_command("sho version"), "show_version");
        assert_eq!(normalize_command("sh ver"), "show_version");
        assert_eq!(normalize_command("sho ip int br"), "show_ip_interface_brief");
        assert_eq!(
            normalize_command("sho ip int br | ex unas"),
            "show_ip_interface_brief_exclude_unassigned"
        );
        assert_eq!(normalize_command("sho line"), "show_line");
    }

    #[test]
    fn normalize_single_word() {
        assert_eq!(normalize_command("dir"), "dir");
    }

    #[test]
    fn normalize_full_commands_unchanged() {
        assert_eq!(normalize_command("show ip interface brief"), "show_ip_interface_brief");
        assert_eq!(normalize_command("show interfaces status"), "show_interfaces_status");
        assert_eq!(normalize_command("show system version"), "show_system_version");
    }

    #[test]
    fn lookup_command_expanded_match() {
        let (key, entry) = lookup_command("cisco_ios", "show ip interface brief");
        assert_eq!(key, "show_ip_interface_brief");
        assert!(entry.is_some());
    }

    #[test]
    fn lookup_command_abbreviated_expanded_match() {
        let (key, entry) = lookup_command("cisco_ios", "sho ip int br | ex unas");
        assert!(entry.is_some());
        assert_eq!(key, "show_ip_interface_brief_exclude_unassigned");
    }

    #[test]
    fn lookup_command_abbreviated_with_existing_expanded() {
        let (key, entry) = lookup_command("cisco_ios", "show ip int br");
        assert!(entry.is_some());
        assert_eq!(key, "show_ip_interface_brief");
    }

    #[test]
    fn normalize_raw_preserves_non_show_abbreviations() {
        assert_eq!(normalize_raw("sho ip int br | ex unas"), "show_ip_int_br_ex_unas");
        assert_eq!(normalize_raw("show running-config"), "show_running_config");
    }

    #[test]
    fn lookup_known_entry() {
        let entry = lookup("cisco_ios", "show_version");
        assert!(entry.is_some());
        let e = entry.unwrap();
        assert_eq!(e.platform, "cisco_ios");
        assert_eq!(e.command_key, "show_version");
        assert!(e.template.ends_with(".textfsm"));
    }

    #[test]
    fn lookup_unknown_entry() {
        assert!(lookup("nonexistent_os", "show_version").is_none());
        assert!(lookup("cisco_ios", "show_magic_unicorn").is_none());
    }

    #[test]
    fn resolve_platform_alias() {
        assert_eq!(resolve_platform("cisco_iosxe"), "cisco_ios");
        assert_eq!(resolve_platform("cisco_ios"), "cisco_ios");
        assert_eq!(resolve_platform("arista_eos"), "arista_eos");
        assert_eq!(resolve_platform("nokia_sros"), "alcatel_sros");
        assert_eq!(resolve_platform("cisco_iosxr"), "cisco_xr");
    }

    #[test]
    fn lookup_via_alias() {
        let entry = lookup("cisco_iosxe", "show_version");
        assert!(entry.is_some());
        let e = entry.unwrap();
        assert_eq!(e.platform, "cisco_ios");
        assert_eq!(e.command_key, "show_version");
    }

    #[test]
    fn lookup_via_nokia_alias() {
        let entry = lookup("nokia_sros", "show_system_cpu");
        assert!(entry.is_some());
        let e = entry.unwrap();
        assert_eq!(e.platform, "alcatel_sros");
    }

    #[test]
    fn lookup_via_iosxr_alias() {
        let entry = lookup("cisco_iosxr", "show_version");
        assert!(entry.is_some());
        let e = entry.unwrap();
        assert_eq!(e.platform, "cisco_xr");
    }

    #[test]
    fn load_template_for_known_entry() {
        let entry = lookup("cisco_ios", "show_version").unwrap();
        let text = load_template_text(entry);
        assert!(text.is_some());
        let content = text.unwrap();
        assert!(content.contains("Value"), "template should contain Value definitions");
        assert!(content.contains("Start"), "template should contain Start state");
    }

    #[test]
    fn registry_has_entries() {
        let reg = registry();
        assert!(reg.len() > 900, "expected 900+ registry entries, got {}", reg.len());
    }
}
