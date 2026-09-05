//! Severity classification for failures and assertion results.
//!
//! Security-sensitive failures (authorization bypass attempts, privacy
//! leakage, verification failures that were silently accepted) must be
//! elevated so reports and CI triage them first. Severity is attached to a
//! [`crate::failure::Failure`] or an assertion result; it describes how
//! seriously a finding should be taken, independent of the failure *category*.

use serde::{Deserialize, Serialize};

/// How seriously a finding should be taken.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational — no defect, or expected behavior observed.
    #[default]
    Info,
    /// Low — cosmetic or non-protocol issue.
    Low,
    /// Medium — a behavioral defect that does not directly threaten security.
    Medium,
    /// High — a security- or integrity-relevant defect.
    High,
    /// Critical — a severe security, privacy, or integrity violation.
    Critical,
}

impl Severity {
    /// All severities in ascending order.
    pub const ALL: [Severity; 5] = [
        Severity::Info,
        Severity::Low,
        Severity::Medium,
        Severity::High,
        Severity::Critical,
    ];

    /// Ascending numeric rank: `Info = 0` … `Critical = 4`.
    pub const fn rank(self) -> u8 {
        match self {
            Severity::Info => 0,
            Severity::Low => 1,
            Severity::Medium => 2,
            Severity::High => 3,
            Severity::Critical => 4,
        }
    }

    /// True for `High` and `Critical` — findings that must never be buried.
    pub const fn is_elevated(self) -> bool {
        matches!(self, Severity::High | Severity::Critical)
    }

    /// Stable machine-readable name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
            Severity::Critical => "critical",
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Severity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "info" => Ok(Severity::Info),
            "low" => Ok(Severity::Low),
            "medium" => Ok(Severity::Medium),
            "high" => Ok(Severity::High),
            "critical" => Ok(Severity::Critical),
            other => Err(format!("unknown severity `{other}`")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranks_are_ascending() {
        let mut ranks = Severity::ALL.iter().map(|s| s.rank());
        let mut prev = ranks.next().unwrap();
        for rank in ranks {
            assert!(rank > prev);
            prev = rank;
        }
    }

    #[test]
    fn only_high_and_critical_are_elevated() {
        assert!(!Severity::Info.is_elevated());
        assert!(!Severity::Low.is_elevated());
        assert!(!Severity::Medium.is_elevated());
        assert!(Severity::High.is_elevated());
        assert!(Severity::Critical.is_elevated());
    }

    #[test]
    fn parses_case_insensitively_and_round_trips() {
        for s in Severity::ALL {
            assert_eq!(s.to_string().parse::<Severity>().unwrap(), s);
            assert_eq!("CRITICAL".parse::<Severity>().unwrap(), Severity::Critical);
        }
        assert!("fatal".parse::<Severity>().is_err());
    }

    #[test]
    fn serde_uses_lowercase_names() {
        assert_eq!(
            serde_json::to_string(&Severity::Critical).unwrap(),
            "\"critical\""
        );
        assert_eq!(
            serde_json::from_str::<Severity>("\"high\"").unwrap(),
            Severity::High
        );
    }
}
