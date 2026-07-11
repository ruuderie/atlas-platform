//! Feature-flag domain types.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Effect of a per-app-instance (or tenant) flag enablement.
///
/// Stored as VARCHAR with CHECK (`grant` | `deny`) in
/// `atlas_flag_instance_enablements.effect` and historically as
/// `flag_overrides.override_type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlagEffect {
    Grant,
    Deny,
}

impl FlagEffect {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Grant => "grant",
            Self::Deny => "deny",
        }
    }
}

impl fmt::Display for FlagEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for FlagEffect {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "grant" => Ok(Self::Grant),
            "deny" => Ok(Self::Deny),
            other => Err(format!("unknown FlagEffect: '{other}'")),
        }
    }
}

impl TryFrom<&str> for FlagEffect {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl TryFrom<String> for FlagEffect {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}
