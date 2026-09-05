//! Stable scenario identifiers.
//!
//! Scenario IDs are **permanent**: `CT-XFER-001` still means the same
//! scenario in ten years. That permanence is why IDs are validated against a
//! strict grammar at construction time — an ID that survives must also be
//! unambiguous in CLI arguments, file names, reports, and issue trackers.
//!
//! Grammar: 1–64 characters of uppercase ASCII letters, digits, and single
//! `-` separators (no leading/trailing `-`, no doubled `-`). Families use a
//! stable prefix: `CT-REG-001`, `CT-XFER-NEG-001`, `CT-PROOF-REPLAY-001`,
//! `REG-2026-001`.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::errors::{Error, Result};

/// A validated, stable scenario identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ScenarioId(String);

impl ScenarioId {
    /// Maximum length of an identifier.
    pub const MAX_LEN: usize = 64;

    /// Validate `raw` against the identifier grammar without constructing.
    pub fn is_valid(raw: &str) -> bool {
        Self::validate(raw).is_ok()
    }

    fn validate(raw: &str) -> std::result::Result<(), String> {
        if raw.is_empty() {
            return Err("must not be empty".to_string());
        }
        if raw.len() > Self::MAX_LEN {
            return Err(format!("must be at most {} characters", Self::MAX_LEN));
        }
        let bytes = raw.as_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            if b.is_ascii_uppercase() || b.is_ascii_digit() {
                continue;
            }
            if b == b'-' {
                let prev_ok =
                    i > 0 && (bytes[i - 1].is_ascii_uppercase() || bytes[i - 1].is_ascii_digit());
                let next_ok = i + 1 < bytes.len()
                    && (bytes[i + 1].is_ascii_uppercase() || bytes[i + 1].is_ascii_digit());
                if !(prev_ok && next_ok) {
                    return Err("`-` must separate two alphanumeric runs".to_string());
                }
                continue;
            }
            return Err(format!(
                "character `{}` not allowed (uppercase A-Z, 0-9, `-`)",
                raw[i..].chars().next().unwrap_or('?')
            ));
        }
        Ok(())
    }

    /// Construct a validated identifier.
    pub fn new(raw: &str) -> Result<Self> {
        Self::validate(raw).map_err(|why| Error::InvalidId(raw.to_string(), why))?;
        Ok(ScenarioId(raw.to_string()))
    }

    /// The identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// First dash-delimited segment (e.g. `CT` or `REG`), or the whole ID.
    pub fn prefix(&self) -> &str {
        self.0.split('-').next().unwrap_or(&self.0)
    }
}

impl fmt::Display for ScenarioId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for ScenarioId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        ScenarioId::new(s)
    }
}

impl AsRef<str> for ScenarioId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Serialize for ScenarioId {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for ScenarioId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        ScenarioId::new(&raw).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_stable_identifier_shapes() {
        for id in [
            "CT-REG-001",
            "CT-DEP-001",
            "CT-MERGE-001",
            "CT-XFER-001",
            "CT-XFER-NEG-001",
            "CT-XFER-ADV-003",
            "CT-PROOF-REPLAY-001",
            "CT-PRIV-001",
            "REG-2026-001",
            "A1",
        ] {
            assert!(ScenarioId::is_valid(id), "{id} should be valid");
        }
    }

    #[test]
    fn rejects_malformed_identifiers() {
        for id in [
            "",
            " ",
            "ct-xfer-001",
            "CT xfer",
            "CT_",
            "-CT-1",
            "CT--1",
            "CT-",
            "CT/XFER",
            &"C".repeat(ScenarioId::MAX_LEN + 1),
        ] {
            assert!(!ScenarioId::is_valid(id), "{id:?} should be invalid");
            assert!(matches!(ScenarioId::new(id), Err(Error::InvalidId(..))));
        }
    }

    #[test]
    fn prefix_and_display() {
        let id = ScenarioId::new("CT-XFER-001").unwrap();
        assert_eq!(id.prefix(), "CT");
        assert_eq!(id.to_string(), "CT-XFER-001");
        assert_eq!(id.as_str(), "CT-XFER-001");
    }

    #[test]
    fn serde_round_trip() {
        let id = ScenarioId::new("CT-XFER-001").unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"CT-XFER-001\"");
        assert_eq!(serde_json::from_str::<ScenarioId>(&json).unwrap(), id);
        assert!(serde_json::from_str::<ScenarioId>("\"ct-lower\"").is_err());
    }
}
