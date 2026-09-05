//! Execution environments.
//!
//! A scenario runs in exactly one declared environment kind (or is written
//! environment-agnostically and matched by capabilities). Environments range
//! from the cheapest trustworthy surface to full deployment:
//!
//! ```text
//! MOCK → SIMULATOR → PROVER → SOROBAN → TESTNET → END_TO_END
//! ```
//!
//! MOCK exists only so runner/registry/assertion logic can be unit-tested
//! with explicit test doubles; a mock is never a substitute for a real
//! environment and mocked proofs are never treated as cryptographically valid.
//! TESTNET is always [`Environment::isolated`]: it requires explicit
//! configuration and must never be required by ordinary CI.

use serde::{Deserialize, Serialize};

/// The kind of system a scenario drives.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EnvironmentKind {
    /// Test doubles only — used by unit tests of this repository's own
    /// infrastructure, never as evidence about the system under test.
    Mock,
    /// `crucible-simulator` deterministic state execution.
    #[default]
    Simulator,
    /// `crucible-prover` proof generation/verification.
    Prover,
    /// Deployed/local Soroban Confidential Token contracts.
    Soroban,
    /// An explicitly configured public test network (isolated).
    Testnet,
    /// Full simulator→prover→contract composition.
    EndToEnd,
}

impl EnvironmentKind {
    /// Stable machine name.
    pub const fn as_str(self) -> &'static str {
        match self {
            EnvironmentKind::Mock => "mock",
            EnvironmentKind::Simulator => "simulator",
            EnvironmentKind::Prover => "prover",
            EnvironmentKind::Soroban => "soroban",
            EnvironmentKind::Testnet => "testnet",
            EnvironmentKind::EndToEnd => "end-to-end",
        }
    }

    /// Whether this kind is an isolated environment that must never run in
    /// ordinary CI without explicit opt-in.
    pub const fn is_isolated(self) -> bool {
        matches!(self, EnvironmentKind::Testnet)
    }
}

impl std::fmt::Display for EnvironmentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for EnvironmentKind {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let normalized = s.trim().to_ascii_lowercase().replace('_', "-");
        match normalized.as_str() {
            "mock" => Ok(EnvironmentKind::Mock),
            "simulator" => Ok(EnvironmentKind::Simulator),
            "prover" => Ok(EnvironmentKind::Prover),
            "soroban" => Ok(EnvironmentKind::Soroban),
            "testnet" => Ok(EnvironmentKind::Testnet),
            "end-to-end" | "e2e" => Ok(EnvironmentKind::EndToEnd),
            other => Err(format!("unknown environment kind `{other}`")),
        }
    }
}

/// A concrete execution environment: kind plus optional identifying details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Environment {
    /// What system this environment drives.
    pub kind: EnvironmentKind,
    /// Optional label (e.g. `futurenet`, `local-soroban`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional reported version of the backing system.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl Environment {
    /// The empty mock environment (unit-testing this repository's own logic).
    pub fn mock() -> Self {
        Environment {
            kind: EnvironmentKind::Mock,
            name: None,
            version: None,
        }
    }

    /// The default simulator environment.
    pub fn simulator() -> Self {
        Environment {
            kind: EnvironmentKind::Simulator,
            name: None,
            version: None,
        }
    }

    /// Build an environment from a kind.
    pub fn new(kind: EnvironmentKind) -> Self {
        Environment {
            kind,
            name: None,
            version: None,
        }
    }

    /// Label this environment.
    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Report the backing system's version.
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Whether this environment is isolated (testnet) and must be opted into.
    pub fn is_isolated(&self) -> bool {
        self.kind.is_isolated()
    }
}

impl Default for Environment {
    fn default() -> Self {
        Environment::simulator()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_parse_and_display_stably() {
        for kind in [
            EnvironmentKind::Mock,
            EnvironmentKind::Simulator,
            EnvironmentKind::Prover,
            EnvironmentKind::Soroban,
            EnvironmentKind::Testnet,
            EnvironmentKind::EndToEnd,
        ] {
            assert_eq!(kind.to_string().parse::<EnvironmentKind>().unwrap(), kind);
        }
        assert_eq!("end_to_end".parse::<EnvironmentKind>().unwrap(), EnvironmentKind::EndToEnd);
        assert!("mainnet".parse::<EnvironmentKind>().is_err());
    }

    #[test]
    fn only_testnet_is_isolated() {
        assert!(EnvironmentKind::Testnet.is_isolated());
        assert!(!EnvironmentKind::Simulator.is_isolated());
        assert!(!Environment::mock().is_isolated());
        assert!(Environment::new(EnvironmentKind::Testnet).is_isolated());
    }

    #[test]
    fn environment_builders_attach_details() {
        let env = Environment::new(EnvironmentKind::Testnet)
            .named("futurenet")
            .with_version("0.9.0-preview");
        assert_eq!(env.kind, EnvironmentKind::Testnet);
        assert_eq!(env.name.as_deref(), Some("futurenet"));
    }

    #[test]
    fn environment_serde_round_trip() {
        let env = Environment::simulator().with_version("1.4.0");
        let json = serde_json::to_string(&env).unwrap();
        assert!(json.contains("\"kind\":\"simulator\""));
        let back: Environment = serde_json::from_str(&json).unwrap();
        assert_eq!(back, env);
    }
}
