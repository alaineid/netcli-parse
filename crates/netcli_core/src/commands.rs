use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandKey {
    ShowVersion,
    ShowInterfacesBrief,
    ShowInventory,
    ShowBgpSummary,
    ShowIpRoute,
    ShowLldpNeighbors,
}

impl CommandKey {
    pub fn slug(self) -> &'static str {
        match self {
            Self::ShowVersion => "show_version",
            Self::ShowInterfacesBrief => "show_interfaces_brief",
            Self::ShowInventory => "show_inventory",
            Self::ShowBgpSummary => "show_bgp_summary",
            Self::ShowIpRoute => "show_ip_route",
            Self::ShowLldpNeighbors => "show_lldp_neighbors",
        }
    }
}

impl fmt::Display for CommandKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.slug())
    }
}

impl FromStr for CommandKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s
            .to_ascii_lowercase()
            .replace(' ', "_")
            .replace('-', "_")
            .as_str()
        {
            "show_version" => Ok(Self::ShowVersion),
            "show_interfaces_brief" | "show_int_brief" => Ok(Self::ShowInterfacesBrief),
            "show_inventory" => Ok(Self::ShowInventory),
            "show_bgp_summary" => Ok(Self::ShowBgpSummary),
            "show_ip_route" => Ok(Self::ShowIpRoute),
            "show_lldp_neighbors" => Ok(Self::ShowLldpNeighbors),
            _ => Err(format!("unknown command key: {s}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_slug_round_trips() {
        for slug in [
            "show_version",
            "show_interfaces_brief",
            "show_inventory",
            "show_bgp_summary",
            "show_ip_route",
            "show_lldp_neighbors",
        ] {
            let ck: CommandKey = slug.parse().unwrap();
            assert_eq!(ck.slug(), slug);
        }
    }

    #[test]
    fn alias_with_spaces_resolves() {
        assert_eq!(
            "show version".parse::<CommandKey>().unwrap(),
            CommandKey::ShowVersion
        );
    }

    #[test]
    fn unknown_command_is_err() {
        assert!("show_magic".parse::<CommandKey>().is_err());
    }
}
