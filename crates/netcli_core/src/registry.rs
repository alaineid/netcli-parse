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

pub(crate) fn lookup(platform: &str, command_key: &str) -> Option<&'static RegistryEntry> {
    registry().get(&(platform.into(), command_key.into()))
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
