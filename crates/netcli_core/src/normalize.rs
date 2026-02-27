use std::collections::HashMap;

/// Normalize vendor-specific field names into a canonical schema.
///
/// Phase 1: pass-through (returns records unchanged).
/// Phase 2: will apply per-CommandKey normalization rules, e.g.
///   - "VERSION" / "version" / "sw_version" → "software_version"
///   - "HOSTNAME" / "hostname" / "host_name" → "hostname"
pub fn normalize(
    _command_key: &str,
    records: Vec<HashMap<String, String>>,
) -> Vec<HashMap<String, String>> {
    // TODO (phase 2): apply canonical field mappings
    records
}
