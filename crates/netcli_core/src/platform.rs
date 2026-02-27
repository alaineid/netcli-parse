use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    CiscoIos,
    CiscoNxos,
    CiscoIosXr,
    JuniperJunos,
    AristaEos,
    DriveNetsDnos,
}

impl Platform {
    pub fn slug(self) -> &'static str {
        match self {
            Self::CiscoIos => "cisco_ios",
            Self::CiscoNxos => "cisco_nxos",
            Self::CiscoIosXr => "cisco_iosxr",
            Self::JuniperJunos => "juniper_junos",
            Self::AristaEos => "arista_eos",
            Self::DriveNetsDnos => "drivenets_dnos",
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.slug())
    }
}

impl FromStr for Platform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().replace('-', "_").as_str() {
            "cisco_ios" | "ios" => Ok(Self::CiscoIos),
            "cisco_nxos" | "nxos" | "nx_os" => Ok(Self::CiscoNxos),
            "cisco_iosxr" | "iosxr" | "ios_xr" => Ok(Self::CiscoIosXr),
            "juniper_junos" | "junos" => Ok(Self::JuniperJunos),
            "arista_eos" | "eos" => Ok(Self::AristaEos),
            "drivenets_dnos" | "dnos" | "drivenets" => Ok(Self::DriveNetsDnos),
            _ => Err(format!("unknown platform: {s}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_slug_round_trips() {
        for slug in [
            "cisco_ios",
            "cisco_nxos",
            "cisco_iosxr",
            "juniper_junos",
            "arista_eos",
            "drivenets_dnos",
        ] {
            let p: Platform = slug.parse().unwrap();
            assert_eq!(p.slug(), slug);
        }
    }

    #[test]
    fn aliases_resolve() {
        assert_eq!("ios".parse::<Platform>().unwrap(), Platform::CiscoIos);
        assert_eq!("nxos".parse::<Platform>().unwrap(), Platform::CiscoNxos);
        assert_eq!("junos".parse::<Platform>().unwrap(), Platform::JuniperJunos);
        assert_eq!("eos".parse::<Platform>().unwrap(), Platform::AristaEos);
        assert_eq!("dnos".parse::<Platform>().unwrap(), Platform::DriveNetsDnos);
    }

    #[test]
    fn unknown_platform_is_err() {
        assert!("foobar".parse::<Platform>().is_err());
    }
}
