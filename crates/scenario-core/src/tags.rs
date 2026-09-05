//! Canonical scenario tags.
//!
//! Tags are the coarse, filterable vocabulary that the CLI and registry use
//! to select scenarios (`crucible-scenarios run --tag adversarial`). They are
//! deliberately coarser than a scenario's category and description: a tag
//! must be a single lowercase word so filters are unambiguous. Standard tags
//! are exported as constants so scenario definitions cannot drift into
//! per-file spellings of the same concept.

use ::serde::{Deserialize, Serialize};
use ::std::collections::BTreeSet;
use ::std::fmt;

/// Canonical, deduplicated, ordered set of scenario tags.
///
/// Stored as a `BTreeSet` so iteration and serialization are deterministic
/// regardless of insertion order — essential for reproducible reports.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Tags(BTreeSet<String>);

/// Standard tag vocabulary. Tags are lowercase and singular.
pub mod standard {
    /// Registration behavior.
    pub const REGISTER: &str = "register";
    /// Deposit behavior.
    pub const DEPOSIT: &str = "deposit";
    /// Confidential balance merge.
    pub const MERGE: &str = "merge";
    /// Confidential transfer.
    pub const TRANSFER: &str = "transfer";
    /// Withdrawal to public balance.
    pub const WITHDRAW: &str = "withdraw";
    /// Complete register→…→withdraw workflows.
    pub const FULL_LIFECYCLE: &str = "full-lifecycle";
    /// Inputs that must be rejected (expected rejection).
    pub const NEGATIVE: &str = "negative";
    /// Attempts to violate system assumptions.
    pub const ADVERSARIAL: &str = "adversarial";
    /// Accidental-disclosure checks.
    pub const PRIVACY: &str = "privacy";
    /// State-transition behavior and snapshots.
    pub const STATE: &str = "state";
    /// Proof construction and verification behavior.
    pub const PROOF: &str = "proof";
    /// Concurrent and racing operations.
    pub const CONCURRENCY: &str = "concurrency";
    /// Protocol conformance.
    pub const CONFORMANCE: &str = "conformance";
    /// Cross-operation invariants.
    pub const INVARIANT: &str = "invariant";
    /// Permanent bug regressions.
    pub const REGRESSION: &str = "regression";
    /// Format/version compatibility.
    pub const COMPATIBILITY: &str = "compatibility";
    /// Load and timing measurement.
    pub const PERFORMANCE: &str = "performance";
    /// Optional testnet execution (never required by ordinary CI).
    pub const TESTNET: &str = "testnet";
    /// Soroban adapter execution.
    pub const SOROBAN: &str = "soroban";
    /// Cross-component integration.
    pub const INTEGRATION: &str = "integration";
    /// Fuzz-targeted boundaries.
    pub const FUZZING: &str = "fuzzing";
    /// Agent-based exploration.
    pub const AGENT: &str = "agent";
}

impl Tags {
    /// An empty tag set.
    pub fn new() -> Self {
        Tags(BTreeSet::new())
    }

    /// Normalize a raw tag: lowercase, trimmed, no internal whitespace.
    fn normalize(tag: &str) -> Option<String> {
        let t = tag.trim().to_ascii_lowercase();
        if t.is_empty() || t.chars().any(|c| c.is_whitespace()) {
            None
        } else {
            Some(t)
        }
    }

    /// Insert a tag (case-insensitive, whitespace-trimmed). Invalid tags are
    /// ignored silently; use [`Tags::try_insert`] when that matters.
    pub fn insert(&mut self, tag: &str) {
        if let Some(normalized) = Tags::normalize(tag) {
            self.0.insert(normalized);
        }
    }

    /// Insert a tag and report whether it was valid.
    pub fn try_insert(&mut self, tag: &str) -> Result<bool, String> {
        match Tags::normalize(tag) {
            Some(normalized) => Ok(self.0.insert(normalized)),
            None => Err(format!("invalid tag `{tag}`: expected one lowercase word")),
        }
    }

    /// Build a set from any number of tags.
    pub fn of<I, S>(tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut set = Tags::new();
        for tag in tags {
            set.insert(tag.as_ref());
        }
        set
    }

    /// Whether `tag` is present (matching is case-insensitive).
    pub fn contains(&self, tag: &str) -> bool {
        Tags::normalize(tag)
            .map(|t| self.0.contains(&t))
            .unwrap_or(false)
    }

    /// Whether any of the given tags is present.
    pub fn has_any<I, S>(&self, tags: I) -> bool
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        tags.into_iter().any(|t| self.contains(t.as_ref()))
    }

    /// Whether all of the given tags are present.
    pub fn has_all<I, S>(&self, tags: I) -> bool
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        tags.into_iter().all(|t| self.contains(t.as_ref()))
    }

    /// Merge another tag set into this one.
    pub fn merge(&mut self, other: &Tags) {
        for tag in &other.0 {
            self.0.insert(tag.clone());
        }
    }

    /// Iterate tags in canonical (sorted) order.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(String::as_str)
    }

    /// Number of tags.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for Tags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let joined = self.iter().collect::<Vec<_>>().join(",");
        f.write_str(&joined)
    }
}

impl FromIterator<String> for Tags {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Tags::of(iter)
    }
}

impl<'de> Deserialize<'de> for Tags {
    /// Deserialize from a sequence of strings, canonicalizing each tag the
    /// same way [`Tags::insert`] does (lowercase, trimmed) so that tags read
    /// back from disk compare equal regardless of source casing.
    fn deserialize<D: ::serde::Deserializer<'de>>(
        deserializer: D,
    ) -> ::std::result::Result<Self, D::Error> {
        let raw = <Vec<String>>::deserialize(deserializer)?;
        Ok(Tags::of(raw))
    }
}

impl<'a> FromIterator<&'a str> for Tags {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        Tags::of(iter)
    }
}

impl IntoIterator for Tags {
    type Item = String;
    type IntoIter = std::collections::btree_set::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalizes_case_and_whitespace() {
        let mut tags = Tags::new();
        tags.insert("  Adversarial ");
        tags.insert("PRIVACY");
        assert!(tags.contains("adversarial"));
        assert!(tags.contains("AdVeRsArIaL"));
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn deduplicates_and_sorts() {
        let tags = Tags::of(["transfer", "transfer", "privacy", "adversarial"]);
        let collected: Vec<_> = tags.iter().collect();
        assert_eq!(collected, vec!["adversarial", "privacy", "transfer"]);
    }

    #[test]
    fn has_any_and_has_all() {
        let tags = Tags::of(["transfer", "negative"]);
        assert!(tags.has_any(["adversarial", "transfer"]));
        assert!(!tags.has_any(["adversarial", "privacy"]));
        assert!(tags.has_all(["transfer", "negative"]));
        assert!(!tags.has_all(["transfer", "privacy"]));
    }

    #[test]
    fn rejects_multiword_tags() {
        let mut tags = Tags::new();
        assert!(tags.try_insert("two words").is_err());
        assert!(tags.try_insert("").is_err());
        assert!(tags.try_insert("single").is_ok());
    }

    #[test]
    fn serde_is_a_sorted_array() {
        let tags = Tags::of(["transfer", "adversarial"]);
        assert_eq!(
            serde_json::to_string(&tags).unwrap(),
            r#"["adversarial","transfer"]"#
        );
        let back: Tags = serde_json::from_str(r#"["PRIVACY","proof"]"#).unwrap();
        assert_eq!(back.iter().collect::<Vec<_>>(), vec!["privacy", "proof"]);
    }

    #[test]
    fn merge_is_union() {
        let mut a = Tags::of(["transfer"]);
        let b = Tags::of(["transfer", "state"]);
        a.merge(&b);
        assert_eq!(a.len(), 2);
    }
}
