//! Capability model: what a scenario requires and what an environment offers.
//!
//! Scenarios declare required capabilities; environments imply capabilities;
//! a run is only valid when the context's capabilities cover the scenario's
//! requirements. This indirection is what lets the same scenario definition
//! run against a simulator today and a Soroban deployment tomorrow — or be
//! skipped cleanly (never silently mis-executed) when a capability is absent.
//!
//! A scenario that requires `testnet` can therefore never accidentally run in
//! ordinary CI: CI contexts simply do not provide the capability.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::environment::EnvironmentKind;
use crate::errors::{Error, Result};

/// A discrete capability a scenario may require or an environment may offer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Capability {
    /// Deterministic Confidential Token state execution (`crucible-simulator`).
    Simulation,
    /// Witness→proof generation (`crucible-prover`).
    ProofProvider,
    /// Proof verification.
    Verifier,
    /// Invocation of deployed/local Soroban Confidential Token contracts.
    SorobanAdapter,
    /// Explicitly configured public testnet execution.
    Testnet,
    /// Observation of emitted events.
    EventObservation,
    /// State snapshot and restore.
    Snapshots,
    /// Genuine concurrent execution (as opposed to merely parallel runs).
    Concurrency,
    /// Protocol-level replay protection.
    ReplayProtection,
    /// A controllable, deterministic clock.
    DeterministicClock,
}

impl Capability {
    /// Stable machine name (kebab-case).
    pub const fn as_str(self) -> &'static str {
        match self {
            Capability::Simulation => "simulation",
            Capability::ProofProvider => "proof-provider",
            Capability::Verifier => "verifier",
            Capability::SorobanAdapter => "soroban-adapter",
            Capability::Testnet => "testnet",
            Capability::EventObservation => "event-observation",
            Capability::Snapshots => "snapshots",
            Capability::Concurrency => "concurrency",
            Capability::ReplayProtection => "replay-protection",
            Capability::DeterministicClock => "deterministic-clock",
        }
    }

    /// The environment kind that natively offers this capability, if any.
    pub const fn home_environment(self) -> Option<EnvironmentKind> {
        match self {
            Capability::Simulation => Some(EnvironmentKind::Simulator),
            Capability::ProofProvider | Capability::Verifier => Some(EnvironmentKind::Prover),
            Capability::SorobanAdapter => Some(EnvironmentKind::Soroban),
            Capability::Testnet => Some(EnvironmentKind::Testnet),
            _ => None,
        }
    }
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Capability {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let normalized = s.trim().to_ascii_lowercase().replace('_', "-");
        match normalized.as_str() {
            "simulation" | "simulator" => Ok(Capability::Simulation),
            "proof-provider" | "prover" => Ok(Capability::ProofProvider),
            "verifier" => Ok(Capability::Verifier),
            "soroban-adapter" | "soroban" => Ok(Capability::SorobanAdapter),
            "testnet" => Ok(Capability::Testnet),
            "event-observation" => Ok(Capability::EventObservation),
            "snapshots" => Ok(Capability::Snapshots),
            "concurrency" => Ok(Capability::Concurrency),
            "replay-protection" => Ok(Capability::ReplayProtection),
            "deterministic-clock" => Ok(Capability::DeterministicClock),
            other => Err(format!("unknown capability `{other}`")),
        }
    }
}

impl Serialize for Capability {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Capability {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        raw.parse::<Capability>().map_err(serde::de::Error::custom)
    }
}

/// Ordered, deduplicated set of capabilities.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Capabilities(BTreeSet<Capability>);

impl Capabilities {
    /// Empty capability set.
    pub fn new() -> Self {
        Capabilities(BTreeSet::new())
    }

    /// Build a set from any number of capabilities.
    pub fn of<I>(caps: I) -> Self
    where
        I: IntoIterator<Item = Capability>,
    {
        Capabilities(caps.into_iter().collect())
    }

    /// Insert a capability.
    pub fn insert(&mut self, cap: Capability) -> bool {
        self.0.insert(cap)
    }

    /// Whether the set contains `cap`.
    pub fn has(&self, cap: Capability) -> bool {
        self.0.contains(&cap)
    }

    /// Whether this set covers every capability in `other` (superset check).
    pub fn covers(&self, other: &Capabilities) -> bool {
        other.0.is_subset(&self.0)
    }

    /// Iterate capabilities in deterministic sorted order.
    pub fn iter(&self) -> impl Iterator<Item = Capability> {
        // BTreeSet<Capability> orders by derived Ord (discriminant order), not
        // by name; iterate by name for stable, readable output.
        let mut names: Vec<Capability> = self.0.iter().copied().collect();
        names.sort_by_key(|c| c.as_str());
        names.into_iter()
    }

    /// Number of capabilities.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The capabilities an environment kind implies when its backing system
    /// is fully functional. A mock environment implies nothing: test doubles
    /// must declare what they fake explicitly.
    pub fn implied_by(kind: EnvironmentKind) -> Self {
        let mut set = Capabilities::new();
        match kind {
            EnvironmentKind::Mock => {}
            EnvironmentKind::Simulator => {
                set.insert(Capability::Simulation);
                set.insert(Capability::EventObservation);
                set.insert(Capability::Snapshots);
            }
            EnvironmentKind::Prover => {
                set.insert(Capability::ProofProvider);
                set.insert(Capability::Verifier);
            }
            EnvironmentKind::Soroban => {
                set.insert(Capability::SorobanAdapter);
                set.insert(Capability::EventObservation);
                set.insert(Capability::Snapshots);
            }
            EnvironmentKind::Testnet => {
                set.insert(Capability::Testnet);
                set.insert(Capability::EventObservation);
                set.insert(Capability::SorobanAdapter);
            }
            EnvironmentKind::EndToEnd => {
                set.insert(Capability::Simulation);
                set.insert(Capability::ProofProvider);
                set.insert(Capability::Verifier);
                set.insert(Capability::SorobanAdapter);
                set.insert(Capability::EventObservation);
                set.insert(Capability::Snapshots);
            }
        }
        set
    }

    /// Check that `required` is fully covered by this set.
    pub fn ensure_covers(&self, required: &Capabilities) -> Result<()> {
        if self.covers(required) {
            Ok(())
        } else {
            let missing: Vec<String> = required
                .0
                .iter()
                .filter(|c| !self.0.contains(c))
                .map(|c| c.as_str().to_string())
                .collect();
            Err(Error::UnavailableCapability(missing.join(", ")))
        }
    }
}

impl Serialize for Capabilities {
    /// Serialize as a name-sorted array, matching [`Capabilities::iter`], so
    /// byte-for-byte output is deterministic across runs and platforms.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let names: Vec<&str> = self.iter().map(|c| c.as_str()).collect();
        names.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Capabilities {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let raw = <Vec<String>>::deserialize(deserializer)?;
        let mut set = Capabilities::new();
        for name in raw {
            let cap: Capability = name.parse().map_err(serde::de::Error::custom)?;
            set.insert(cap);
        }
        Ok(set)
    }
}

impl std::fmt::Display for Capabilities {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let joined = self.iter().map(|c| c.as_str()).collect::<Vec<_>>().join(",");
        f.write_str(&joined)
    }
}

impl FromIterator<Capability> for Capabilities {
    fn from_iter<I: IntoIterator<Item = Capability>>(iter: I) -> Self {
        Capabilities::of(iter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_names_parse_and_round_trip() {
        for cap in [
            Capability::Simulation,
            Capability::ProofProvider,
            Capability::Verifier,
            Capability::SorobanAdapter,
            Capability::Testnet,
            Capability::EventObservation,
            Capability::Snapshots,
            Capability::Concurrency,
            Capability::ReplayProtection,
            Capability::DeterministicClock,
        ] {
            assert_eq!(cap.to_string().parse::<Capability>().unwrap(), cap);
            assert_eq!(serde_json::to_string(&cap).unwrap(), format!("\"{}\"", cap.as_str()));
        }
        assert_eq!("simulator".parse::<Capability>().unwrap(), Capability::Simulation);
        assert_eq!("prover".parse::<Capability>().unwrap(), Capability::ProofProvider);
        assert!("fast-ai".parse::<Capability>().is_err());
    }

    #[test]
    fn covers_is_a_superset_check() {
        let full = Capabilities::of([Capability::Simulation, Capability::ProofProvider]);
        let subset = Capabilities::of([Capability::Simulation]);
        assert!(full.covers(&subset));
        assert!(!subset.covers(&full));
        assert!(full.ensure_covers(&subset).is_ok());
        assert!(subset.ensure_covers(&full).is_err());
    }

    #[test]
    fn environments_imply_capabilities() {
        let sim = Capabilities::implied_by(EnvironmentKind::Simulator);
        assert!(sim.has(Capability::Simulation));
        assert!(sim.has(Capability::Snapshots));
        assert!(!sim.has(Capability::ProofProvider));

        let e2e = Capabilities::implied_by(EnvironmentKind::EndToEnd);
        assert!(e2e.has(Capability::Simulation));
        assert!(e2e.has(Capability::ProofProvider));
        assert!(e2e.has(Capability::Verifier));

        // A testnet scenario can never silently run on a simulator context.
        let testnet = Capabilities::of([Capability::Testnet]);
        assert!(sim.ensure_covers(&testnet).is_err());
    }

    #[test]
    fn iteration_is_deterministic_and_sorted_by_name() {
        let set = Capabilities::of([Capability::Testnet, Capability::Simulation, Capability::ProofProvider]);
        let names: Vec<_> = set.iter().map(|c| c.as_str()).collect();
        assert_eq!(names, vec!["proof-provider", "simulation", "testnet"]);
        assert_eq!(serde_json::to_string(&set).unwrap(),
            r#"["proof-provider","simulation","testnet"]"#);
    }

    #[test]
    fn mock_implies_nothing() {
        assert!(Capabilities::implied_by(EnvironmentKind::Mock).is_empty());
    }
}
