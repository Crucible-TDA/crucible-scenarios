//! Declarative expectations.
//!
//! An [`Expectation`] states what a scenario expects to *observe*, phrased as
//! a checkable claim: this operation succeeds, that operation is rejected
//! (an *expected rejection*, not a failure), a replay is refused, an
//! invariant holds, a value stays undisclosed. Expectations are the contract
//! between the scenario definition and the system under test, and they are
//! deliberately decoupled from how the check is computed — assertion
//! execution and invariant evaluation live in their own crates, so a
//! scenario's intent survives even when an adapter changes underneath it.
//!
//! Crucially, "expected rejection" is a first-class kind here. A negative
//! scenario *passes* when the system rejects what should be rejected; only an
//! *unexpected* acceptance (or an unexpected rejection) is a failure. The
//! runner classifies results against these expectations rather than treating
//! every error as a failure.

use serde::{Deserialize, Serialize};

use crate::operation::OperationId;

/// One checkable claim about a scenario run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Expectation {
    /// Stable key within the scenario (e.g. `expect-op-2-success`).
    pub id: String,
    /// Human-readable statement of what is expected and why.
    pub description: String,
    /// The claim itself.
    pub kind: ExpectationKind,
}

/// The kinds of claims a scenario can make.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpectationKind {
    /// The referenced operation must complete successfully.
    OperationSucceeds {
        /// Operation the claim is about.
        operation: OperationId,
    },
    /// The referenced operation must be rejected — and that rejection is the
    /// point of the scenario. `reason` optionally names the rejection the
    /// protocol should give, so an unrelated error is not mistaken for the
    /// expected one.
    OperationRejected {
        /// Operation that must be refused.
        operation: OperationId,
        /// Optional expected rejection reason/code.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
    /// Re-using the referenced operation (or its proof) must be refused.
    ReplayRejected {
        /// Operation that must not execute twice.
        operation: OperationId,
    },
    /// A named cross-operation invariant must hold across the whole scenario.
    InvariantHolds {
        /// Invariant identifier (registry key, e.g. `conservation`).
        invariant: String,
    },
    /// The named observation must never reach a persistent or logged surface.
    NotDisclosed {
        /// Observation key that must stay private.
        observation: String,
    },
}

impl Expectation {
    /// Build an expectation.
    pub fn new(id: impl Into<String>, description: impl Into<String>, kind: ExpectationKind) -> Self {
        Expectation {
            id: id.into(),
            description: description.into(),
            kind,
        }
    }

    /// Convenience: the named operation succeeds.
    pub fn succeeds(id: impl Into<String>, description: impl Into<String>, operation: OperationId) -> Self {
        Self::new(id, description, ExpectationKind::OperationSucceeds { operation })
    }

    /// Convenience: the named operation is rejected, optionally with a reason.
    pub fn rejected(
        id: impl Into<String>,
        description: impl Into<String>,
        operation: OperationId,
        reason: Option<impl Into<String>>,
    ) -> Self {
        Self::new(
            id,
            description,
            ExpectationKind::OperationRejected {
                operation,
                reason: reason.map(Into::into),
            },
        )
    }

    /// The operation this expectation is about, when it is about one.
    pub fn referenced_operation(&self) -> Option<&OperationId> {
        match &self.kind {
            ExpectationKind::OperationSucceeds { operation }
            | ExpectationKind::OperationRejected { operation, .. }
            | ExpectationKind::ReplayRejected { operation } => Some(operation),
            ExpectationKind::InvariantHolds { .. } | ExpectationKind::NotDisclosed { .. } => None,
        }
    }

    /// Whether this expectation is about an operation being refused.
    pub fn expects_rejection(&self) -> bool {
        matches!(self.kind, ExpectationKind::OperationRejected { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn op(id: &str) -> OperationId {
        OperationId::new(id).unwrap()
    }

    #[test]
    fn expectation_kinds_round_trip_through_json() {
        let expectations = vec![
            Expectation::succeeds("e1", "deposit succeeds", op("op-1")),
            Expectation::rejected("e2", "insufficient balance rejected", op("op-2"), Some("insufficient-balance")),
            Expectation::new(
                "e3",
                "transfer amount stays private",
                ExpectationKind::NotDisclosed { observation: "op-3.transfer.amount".to_string() },
            ),
            Expectation::new(
                "e4",
                "conservation holds",
                ExpectationKind::InvariantHolds { invariant: "conservation".to_string() },
            ),
            Expectation::new("e5", "no replay", ExpectationKind::ReplayRejected { operation: op("op-4") }),
        ];
        for e in &expectations {
            let json = serde_json::to_string(e).unwrap();
            let back: Expectation = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, e);
        }
    }

    #[test]
    fn rejection_is_distinguishable_from_success() {
        let rejected = Expectation::rejected("e", "must reject", op("op-1"), None::<String>);
        assert!(rejected.expects_rejection());
        assert_eq!(rejected.referenced_operation().unwrap().as_str(), "op-1");

        let ok = Expectation::succeeds("e", "must succeed", op("op-1"));
        assert!(!ok.expects_rejection());
    }

    #[test]
    fn invariant_and_privacy_expectations_reference_no_operation() {
        let invariant = Expectation::new("e", "invariant", ExpectationKind::InvariantHolds {
            invariant: "ownership".to_string(),
        });
        assert!(invariant.referenced_operation().is_none());
        let privacy = Expectation::new("e", "privacy", ExpectationKind::NotDisclosed {
            observation: "balance".to_string(),
        });
        assert!(privacy.referenced_operation().is_none());
    }
}
