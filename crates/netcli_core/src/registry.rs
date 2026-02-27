use crate::commands::CommandKey;
use crate::platform::Platform;

/// Resolves (platform, command_key) to a template resource path (relative to
/// the `resources/templates/` directory).
///
/// Phase 2 will load the mapping from `resources/registry.json` and actually
/// read template files. For now this returns a deterministic path that follows
/// the convention `<platform_slug>/<command_slug>.textfsm`.
pub fn template_path(platform: Platform, command_key: CommandKey) -> String {
    format!(
        "{}/{}.textfsm",
        platform.slug(),
        command_key.slug()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_follows_convention() {
        let path = template_path(Platform::CiscoIos, CommandKey::ShowVersion);
        assert_eq!(path, "cisco_ios/show_version.textfsm");
    }
}
