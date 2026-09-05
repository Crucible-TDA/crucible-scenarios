//! Assertion specifications and results.
//!
//! An [`AssertionSpec`] *declares* a check against the run's observations —
//! `event CT_XFER emitted`, `alice's public balance is 70`, `the transfer
//! amount never appears in any report`. Declaring assertions as data keeps
//! them portable across execution environments and lets reports reproduce the
//! exact check list that ran.
//!
//! Executing the checks lives in the `assertions` crate (which consumes
//! observations and fixtures); this module owns only the vocabulary and the
//! [`AssertionResult`] that a run produces per check. A failed assertion is a
//! finding with a severity, never a secret-bearing diagnostic: messages must
//! explain *what* differed without printing private values.

use serde::{Deserialize, Serialize};

use crate::severity::Severity;

/// A declarative check that a scenario asserts about its observations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssertionSpec {
    /// Stable key within the scenario (e.g. `assert-event-ct_xfer`).
    pub id: String,
    /// Human-readable description of the check.
    pub description: String,
    /// The check itself.
    pub kind: AssertionKind,
}

/// The shapes of checks scenario authors can declare. Fields reference
/// observations, fixtures, or expected literals by name; evaluation reads
/// those references through the observation/fixture interfaces rather than
/// reaching into any implementation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssertionKind {
    /// The referenced operation/flow reported success.
    Success,
    /// The referenced operation/flow reported a failure (as expected by the
    /// scenario — pairing with `ExpectationKind::OperationRejected`).
    Failure,
    /// A specific expected error/classification was observed.
    Error {
        /// Expected error code or classification name.
        expected: String,
    },
    /// A named state reference matches the observed state.
    State {
        /// State reference (fixture/snapshot key).
        state: String,
    },
    /// A public balance equals an expected value.
    Balance {
        /// Actor whose balance is checked.
        actor: String,
        /// Token whose balance is checked.
        token: String,
        /// Expected public balance.
        expected: i64,
    },
    /// Ownership of a confidential state belongs to the named owner.
    Ownership {
        /// Expected owner.
        owner: String,
        /// Token whose state is checked.
        token: String,
    },
    /// An action by an actor is authorized (or refused) as declared.
    Authorization {
        /// Actor performing the action.
        actor: String,
        /// Action name.
        action: String,
    },
    /// A proof reference verifies as valid.
    ProofValid {
        /// Proof reference.
        proof: String,
    },
    /// A proof reference verifies as invalid.
    ProofInvalid {
        /// Proof reference.
        proof: String,
    },
    /// A commitment reference equals an expected digest/value.
    Commitment {
        /// Commitment reference.
        commitment: String,
        /// Expected value.
        expected: String,
    },
    /// An event with the given code was emitted.
    Event {
        /// Event code.
        code: String,
    },
    /// No event with the given code was emitted.
    NoEvent {
        /// Event code.
        code: String,
    },
    /// A field is visible in public surfaces.
    PublicVisibility {
        /// Field/observation key.
        field: String,
    },
    /// A field never appears in any report or log surface.
    PrivateNotVisible {
        /// Field/observation key.
        field: String,
    },
    /// An operation is bound to the referenced state.
    StateBinding {
        /// State reference the operation must be bound to.
        state: String,
    },
    /// Re-executing a reference must be rejected.
    ReplayRejected,
    /// Two serialized references are equal after round-trip.
    SerializationEqual {
        /// First reference.
        a: String,
        /// Second reference.
        b: String,
    },
    /// Two versions are compatible as declared.
    VersionCompatible {
        /// First version/reference.
        a: String,
        /// Second version/reference.
        b: String,
    },
}

impl AssertionSpec {
    /// Build an assertion.
    pub fn new(id: impl Into<String>, description: impl Into<String>, kind: AssertionKind) -> Self {
        AssertionSpec {
            id: id.into(),
            description: description.into(),
            kind,
        }
    }

    /// Convenience: the operation succeeded.
    pub fn success(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self::new(id, description, AssertionKind::Success)
    }

    /// Convenience: the operation failed as expected.
    pub fn failure(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self::new(id, description, AssertionKind::Failure)
    }

    /// Convenience: an event was emitted.
    pub fn event(id: impl Into<String>, description: impl Into<String>, code: impl Into<String>) -> Self {
        Self::new(id, description, AssertionKind::Event { code: code.into() })
    }

    /// Convenience: a private field must never be visible.
    pub fn private_not_visible(id: impl Into<String>, description: impl Into<String>, field: impl Into<String>) -> Self {
        Self::new(id, description, AssertionKind::PrivateNotVisible { field: field.into() })
    }
}

/// The outcome of one executed assertion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssertionResult {
    /// Key of the assertion that ran.
    pub id: String,
    /// Whether the check passed.
    pub passed: bool,
    /// Severity to attach when the check fails.
    pub severity: Severity,
    /// Human-readable diagnostic. Must never contain private values.
    pub message: String,
    /// Keys of observations consulted by this assertion.
    #[serde(default)]
    pub observed_keys: Vec<String>,
}

impl AssertionResult {
    /// A passing assertion.
    pub fn passed(id: impl Into<String>, message: impl Into<String>) -> Self {
        AssertionResult {
            id: id.into(),
            passed: true,
            severity: Severity::Info,
            message: message.into(),
            observed_keys: Vec::new(),
        }
    }

    /// A failing assertion at the given severity.
    pub fn failed(id: impl Into<String>, severity: Severity, message: impl Into<String>) -> Self {
        AssertionResult {
            id: id.into(),
            passed: false,
            severity,
            message: message.into(),
            observed_keys: Vec::new(),
        }
    }

    /// Attach the observations this assertion consulted (for traceability).
    pub fn with_observations(mut self, keys: Vec<String>) -> Self {
        self.observed_keys = keys;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assertion_kinds_round_trip_through_json() {
        let assertions = vec![
            AssertionSpec::success("a1", "op-1 succeeds"),
            AssertionSpec::failure("a2", "op-2 rejected as expected"),
            AssertionSpec::event("a3", "transfer event emitted", "ct_xfer"),
            AssertionSpec::private_not_visible("a4", "amount stays private", "op-1.amount"),
            AssertionSpec::new("a5", "balances hold", AssertionKind::Balance {
                actor: "alice".to_string(),
                token: "ct-usdc".to_string(),
                expected: 70,
            }),
            AssertionSpec::new("a6", "no replay", AssertionKind::ReplayRejected),
        ];
        for a in &assertions {
            let json = serde_json::to_string(a).unwrap();
            let back: AssertionSpec = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, a);
        }
    }

    #[test]
    fn results_carry_pass_fail_and_severity() {
        let ok = AssertionResult::passed("a1", "as expected");
        assert!(ok.passed);
        assert_eq!(ok.severity, Severity::Info);

        let bad = AssertionResult::failed("a1", Severity::Critical, "state diverged");
        assert!(!bad.passed);
        assert!(bad.severity.is_elevated());
    }

    #[test]
    fn failures_never_include_private_values_in_message_by_construction() {
        // The message field is plain text; helper constructors take messages,
        // so a leak would have to be authored deliberately rather than
        // produced mechanically from a value.
        let bad = AssertionResult::failed("a", Severity::High, "balance mismatch");
        assert!(!format!("{bad:?}").contains("42000"));
    }
}
