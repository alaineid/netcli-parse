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
        other => other,
    }
}

pub(crate) fn lookup(platform: &str, command_key: &str) -> Option<&'static RegistryEntry> {
    let canonical = resolve_platform(platform);
    registry().get(&(canonical.into(), command_key.into()))
}

pub(crate) fn normalize_command(command: &str) -> String {
    command
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
        .to_ascii_lowercase()
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
    fn normalize_preserves_hyphens() {
        assert_eq!(normalize_command("show boot-config"), "show_boot-config");
    }

    #[test]
    fn normalize_single_word() {
        assert_eq!(normalize_command("dir"), "dir");
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
