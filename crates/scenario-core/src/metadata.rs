//! Scenario metadata: what a scenario *is* before it runs.
//!
//! [`ScenarioMetadata`] carries identity (id, name, description), coarse
//! classification ([`Category`], [`Tags`]), and compatibility-version pins
//! (protocol, circuit, prover, simulator). It deliberately excludes anything
//! runtime: operations, expectations, assertions, seeds, timeouts, and
//! capability requirements belong on [`crate::scenario::Scenario`], and
//! everything observed during a run belongs in outcomes and observations.

use serde::{Deserialize, Serialize};

use crate::errors::{Error, Result};
use crate::scenario_id::ScenarioId;
use crate::tags::Tags;

/// Coarse scenario classification used by the registry and CLI
/// (`--category`), mirroring the scenario families under `scenarios/`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    /// A scenario whose inputs are expected to succeed.
    #[default]
    HappyPath,
    /// A scenario whose inputs must be rejected (expected rejection).
    Negative,
    /// An attempt to violate a system assumption.
    Adversarial,
    /// Accidental disclosure of private information.
    Privacy,
    /// State-transition, snapshot, and rollback behavior.
    State,
    /// Proof construction and verification behavior.
    Proof,
    /// Concurrent and racing operations.
    Concurrency,
    /// Cross-component integration.
    Integration,
    /// Format/version compatibility.
    Compatibility,
    /// Load, timing, and scale behavior.
    Performance,
    /// Protocol conformance against stable contracts.
    Conformance,
    /// Permanent regression coverage for fixed bugs.
    Regression,
    /// Cross-operation invariants.
    Invariant,
    /// Agent-based exploration (secondary).
    Agent,
}

impl Category {
    /// All categories.
    pub const ALL: [Category; 14] = [
        Category::HappyPath,
        Category::Negative,
        Category::Adversarial,
        Category::Privacy,
        Category::State,
        Category::Proof,
        Category::Concurrency,
        Category::Integration,
        Category::Compatibility,
        Category::Performance,
        Category::Conformance,
        Category::Regression,
        Category::Invariant,
        Category::Agent,
    ];

    /// Stable kebab-case machine name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Category::HappyPath => "happy-path",
            Category::Negative => "negative",
            Category::Adversarial => "adversarial",
            Category::Privacy => "privacy",
            Category::State => "state",
            Category::Proof => "proof",
            Category::Concurrency => "concurrency",
            Category::Integration => "integration",
            Category::Compatibility => "compatibility",
            Category::Performance => "performance",
            Category::Conformance => "conformance",
            Category::Regression => "regression",
            Category::Invariant => "invariant",
            Category::Agent => "agent",
        }
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Category {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let normalized = s.trim().to_ascii_lowercase().replace('_', "-");
        Category::ALL
            .iter()
            .find(|c| c.as_str() == normalized)
            .copied()
            .ok_or_else(|| format!("unknown category `{s}`"))
    }
}

/// Human- and machine-readable identity of a scenario.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioMetadata {
    /// Stable, permanent scenario identifier.
    pub id: ScenarioId,
    /// Short human-readable name, e.g. "confidential transfer, sufficient balance".
    pub name: String,
    /// Longer description of intent, setup, and expected behavior.
    pub description: String,
    /// Coarse classification.
    pub category: Category,
    /// Filterable tags (see [`crate::tags::standard`]).
    #[serde(default)]
    pub tags: Tags,
    /// Confidential Token protocol version this scenario targets.
    pub protocol_version: String,
    /// Circuit version the scenario's proofs target, when pinned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub circuit_version: Option<String>,
    /// Prover version the scenario targets, when pinned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prover_version: Option<String>,
    /// Simulator version the scenario targets, when pinned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub simulator_version: Option<String>,
    /// Pointers to supporting documentation or issue references.
    #[serde(default)]
    pub references: Vec<String>,
}

impl ScenarioMetadata {
    /// Build metadata with the mandatory identity fields; versions default to
    /// `"unpinned"` and can be refined with the `with_*` builders.
    pub fn new(
        id: ScenarioId,
        name: impl Into<String>,
        description: impl Into<String>,
        category: Category,
    ) -> Result<Self> {
        let name = name.into();
        let description = description.into();
        if name.trim().is_empty() {
            return Err(Error::InvalidMetadata("name must not be empty".into()));
        }
        if description.trim().is_empty() {
            return Err(Error::InvalidMetadata(
                "description must not be empty".into(),
            ));
        }
        Ok(ScenarioMetadata {
            id,
            name,
            description,
            category,
            tags: Tags::new(),
            protocol_version: "unpinned".to_string(),
            circuit_version: None,
            prover_version: None,
            simulator_version: None,
            references: Vec::new(),
        })
    }

    /// Attach filterable tags.
    pub fn with_tags(mut self, tags: Tags) -> Self {
        self.tags = tags;
        self
    }

    /// Pin the protocol version this scenario validates against.
    pub fn with_protocol_version(mut self, version: impl Into<String>) -> Self {
        self.protocol_version = version.into();
        self
    }

    /// Pin the circuit version, when proofs reference a circuit.
    pub fn with_circuit_version(mut self, version: impl Into<String>) -> Self {
        self.circuit_version = Some(version.into());
        self
    }

    /// Pin the prover version, when proofs come from a prover.
    pub fn with_prover_version(mut self, version: impl Into<String>) -> Self {
        self.prover_version = Some(version.into());
        self
    }

    /// Pin the simulator version this scenario drives.
    pub fn with_simulator_version(mut self, version: impl Into<String>) -> Self {
        self.simulator_version = Some(version.into());
        self
    }

    /// Add supporting references (docs, issues).
    pub fn with_reference(mut self, reference: impl Into<String>) -> Self {
        self.references.push(reference.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn category_names_round_trip() {
        for c in Category::ALL {
            assert_eq!(Category::from_str(c.as_str()).unwrap(), c);
            assert_eq!(
                Category::from_str(&c.as_str().to_uppercase().replace('-', "_")).unwrap(),
                c
            );
        }
        assert!(Category::from_str("nope").is_err());
    }

    #[test]
    fn metadata_requires_identity_fields() {
        let id = ScenarioId::new("CT-XFER-001").unwrap();
        assert!(ScenarioMetadata::new(id.clone(), "", "d", Category::HappyPath).is_err());
        assert!(ScenarioMetadata::new(id.clone(), "n", "", Category::HappyPath).is_err());
        let m = ScenarioMetadata::new(id, "n", "d", Category::HappyPath).unwrap();
        assert_eq!(m.protocol_version, "unpinned");
    }

    #[test]
    fn builders_attach_versions_and_tags() {
        let id = ScenarioId::new("CT-XFER-001").unwrap();
        let m = ScenarioMetadata::new(id, "xfer", "desc", Category::HappyPath)
            .unwrap()
            .with_tags(Tags::of(["transfer"]))
            .with_protocol_version("1.2.0")
            .with_circuit_version("ct-v3")
            .with_prover_version("2.0.0")
            .with_simulator_version("1.4.0")
            .with_reference("docs/simulator-integration.md");
        assert!(m.tags.contains("transfer"));
        assert_eq!(m.circuit_version.as_deref(), Some("ct-v3"));
        assert_eq!(m.references.len(), 1);
    }

    #[test]
    fn metadata_serde_round_trip() {
        let id = ScenarioId::new("CT-PRIV-001").unwrap();
        let m = ScenarioMetadata::new(id, "privacy", "no leakage", Category::Privacy)
            .unwrap()
            .with_tags(Tags::of(["privacy", "proof"]))
            .with_protocol_version("1.0.0");
        let json = serde_json::to_string(&m).unwrap();
        let back: ScenarioMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(back, m);
        // category serializes kebab-case
        assert!(json.contains("\"category\":\"privacy\""));
        assert!(json.contains("\"id\":\"CT-PRIV-001\""));
    }
}
