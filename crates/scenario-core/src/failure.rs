//! Failure classification.
//!
//! Every outcome is classified. [`FailureCategory`] says *what kind* of
//! problem a finding is; [`LifecycleStage`] says *where* it happened;
//! [`Severity`] says how seriously to take it; [`Failure`] bundles all three
//! with a diagnostic. Classification is what lets a report answer "were there
//! privacy failures?" without re-reading prose, and what stops an expected
//! rejection in a negative scenario from being counted as a defect.

use serde::{Deserialize, Serialize};

use crate::severity::Severity;

/// What kind of problem a failure is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FailureCategory {
    /// The scenario definition itself is invalid (a test-harness defect).
    ScenarioDefinitionError,
    /// A fixture was missing, stale, or malformed (a test-harness defect).
    FixtureError,
    /// The execution environment misbehaved or was misconfigured.
    EnvironmentError,
    /// The system rejected input exactly as a negative scenario expected;
    /// recorded only when surrounding expectations still failed.
    ExpectedRejection,
    /// The system accepted input that should have been rejected.
    UnexpectedAcceptance,
    /// A declared assertion did not hold.
    AssertionFailure,
    /// A declared cross-operation invariant did not hold.
    InvariantFailure,
    /// Proof construction failed unexpectedly.
    ProofFailure,
    /// A proof that should verify failed, or one that should fail verified.
    VerificationFailure,
    /// State diverged from the expected transition.
    StateFailure,
    /// An unauthorized actor/action mutated protected state.
    AuthorizationFailure,
    /// Private information became observable through infrastructure.
    PrivacyFailure,
    /// Serialization did not round-trip as declared.
    SerializationFailure,
    /// Version/format compatibility broke where it was promised (or was
    /// silently accepted where it was not).
    CompatibilityFailure,
    /// The scenario exceeded its timeout.
    Timeout,
    /// The scenario was cancelled.
    Cancellation,
    /// Harness infrastructure failed (I/O, scheduler, orchestration).
    InfrastructureFailure,
    /// Nothing else fits.
    UnknownFailure,
}

impl FailureCategory {
    /// Stable machine name.
    pub const fn as_str(self) -> &'static str {
        match self {
            FailureCategory::ScenarioDefinitionError => "SCENARIO_DEFINITION_ERROR",
            FailureCategory::FixtureError => "FIXTURE_ERROR",
            FailureCategory::EnvironmentError => "ENVIRONMENT_ERROR",
            FailureCategory::ExpectedRejection => "EXPECTED_REJECTION",
            FailureCategory::UnexpectedAcceptance => "UNEXPECTED_ACCEPTANCE",
            FailureCategory::AssertionFailure => "ASSERTION_FAILURE",
            FailureCategory::InvariantFailure => "INVARIANT_FAILURE",
            FailureCategory::ProofFailure => "PROOF_FAILURE",
            FailureCategory::VerificationFailure => "VERIFICATION_FAILURE",
            FailureCategory::StateFailure => "STATE_FAILURE",
            FailureCategory::AuthorizationFailure => "AUTHORIZATION_FAILURE",
            FailureCategory::PrivacyFailure => "PRIVACY_FAILURE",
            FailureCategory::SerializationFailure => "SERIALIZATION_FAILURE",
            FailureCategory::CompatibilityFailure => "COMPATIBILITY_FAILURE",
            FailureCategory::Timeout => "TIMEOUT",
            FailureCategory::Cancellation => "CANCELLATION",
            FailureCategory::InfrastructureFailure => "INFRASTRUCTURE_FAILURE",
            FailureCategory::UnknownFailure => "UNKNOWN_FAILURE",
        }
    }

    /// The severity a failure of this category deserves by default.
    pub const fn default_severity(self) -> Severity {
        match self {
            // Security-relevant categories must surface loudly.
            FailureCategory::UnexpectedAcceptance | FailureCategory::PrivacyFailure => Severity::Critical,
            FailureCategory::AuthorizationFailure | FailureCategory::VerificationFailure => Severity::High,
            // Integrity and behavior defects.
            FailureCategory::InvariantFailure
            | FailureCategory::AssertionFailure
            | FailureCategory::ProofFailure
            | FailureCategory::StateFailure
            | FailureCategory::CompatibilityFailure => Severity::Medium,
            // Harness/definition problems: real defects, but not findings
            // about the system under test.
            FailureCategory::ScenarioDefinitionError
            | FailureCategory::FixtureError
            | FailureCategory::EnvironmentError
            | FailureCategory::InfrastructureFailure
            | FailureCategory::SerializationFailure => Severity::Low,
            FailureCategory::Timeout => Severity::Medium,
            FailureCategory::ExpectedRejection => Severity::Low,
            FailureCategory::Cancellation | FailureCategory::UnknownFailure => Severity::Info,
        }
    }

    /// Whether this category is security-sensitive and must never be buried
    /// or silently retried away.
    pub const fn is_security_sensitive(self) -> bool {
        matches!(
            self,
            FailureCategory::UnexpectedAcceptance
                | FailureCategory::AuthorizationFailure
                | FailureCategory::PrivacyFailure
                | FailureCategory::VerificationFailure
        )
    }
}

impl std::fmt::Display for FailureCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Where in the scenario lifecycle a failure happened.
///
/// Mirrors the documented lifecycle: discover → validate → prepare →
/// initialize → execute → observe → assert → validate invariants → classify →
/// report → cleanup. Knowing the stage keeps "the scenario never ran because
/// validation failed" distinct from "the system under test misbehaved", which
/// is essential for honest reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LifecycleStage {
    /// Scenario discovery/registry lookup.
    Discover,
    /// Scenario definition validation.
    Validate,
    /// Environment/fixture preparation.
    Prepare,
    /// Context and initial-state initialization.
    Initialize,
    /// Operation execution against the system under test.
    Execute,
    /// Observation capture.
    Observe,
    /// Assertion evaluation.
    Assert,
    /// Invariant evaluation across the run.
    Invariants,
    /// Outcome classification.
    Classify,
    /// Reporting.
    Report,
    /// Cleanup/teardown.
    Cleanup,
}

impl LifecycleStage {
    /// Stable machine name.
    pub const fn as_str(self) -> &'static str {
        match self {
            LifecycleStage::Discover => "DISCOVER",
            LifecycleStage::Validate => "VALIDATE",
            LifecycleStage::Prepare => "PREPARE",
            LifecycleStage::Initialize => "INITIALIZE",
            LifecycleStage::Execute => "EXECUTE",
            LifecycleStage::Observe => "OBSERVE",
            LifecycleStage::Assert => "ASSERT",
            LifecycleStage::Invariants => "INVARIANTS",
            LifecycleStage::Classify => "CLASSIFY",
            LifecycleStage::Report => "REPORT",
            LifecycleStage::Cleanup => "CLEANUP",
        }
    }
}

impl std::fmt::Display for LifecycleStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A classified failure: category + stage + severity + safe diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Failure {
    /// What kind of problem this is.
    pub category: FailureCategory,
    /// How seriously to take it.
    pub severity: Severity,
    /// Where it happened.
    pub stage: LifecycleStage,
    /// Human-readable diagnostic — never contains private values.
    pub message: String,
    /// Operation key the failure is about, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    /// Permanent regression ID when this failure is a known regression.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regression_id: Option<String>,
    /// Optional extra context lines (safe, public only).
    #[serde(default)]
    pub detail: Vec<String>,
}

impl Failure {
    /// Build a classified failure, assigning the category's default severity.
    pub fn new(
        category: FailureCategory,
        stage: LifecycleStage,
        message: impl Into<String>,
    ) -> Self {
        Failure {
            category,
            severity: category.default_severity(),
            stage,
            message: message.into(),
            operation: None,
            regression_id: None,
            detail: Vec::new(),
        }
    }

    /// Override the severity (usually only to raise it).
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Name the operation the failure is about.
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }

    /// Link a permanent regression record.
    pub fn with_regression(mut self, regression_id: impl Into<String>) -> Self {
        self.regression_id = Some(regression_id.into());
        self
    }

    /// Add a safe context line.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail.push(detail.into());
        self
    }

    /// Whether this failure is security-sensitive.
    pub fn is_security_sensitive(&self) -> bool {
        self.category.is_security_sensitive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_names_serialize_to_stable_codes() {
        for c in [
            FailureCategory::ScenarioDefinitionError,
            FailureCategory::UnexpectedAcceptance,
            FailureCategory::AuthorizationFailure,
            FailureCategory::PrivacyFailure,
            FailureCategory::Timeout,
            FailureCategory::UnknownFailure,
        ] {
            assert_eq!(serde_json::to_string(&c).unwrap(), format!("\"{}\"", c.as_str()));
        }
        assert_eq!(FailureCategory::as_str(FailureCategory::UnexpectedAcceptance), "UNEXPECTED_ACCEPTANCE");
    }

    #[test]
    fn security_sensitive_categories_get_elevated_defaults() {
        assert!(FailureCategory::UnexpectedAcceptance.default_severity().is_elevated());
        assert!(FailureCategory::PrivacyFailure.default_severity().is_elevated());
        assert!(FailureCategory::AuthorizationFailure.default_severity().is_elevated());
        assert!(FailureCategory::VerificationFailure.default_severity().is_elevated());
        assert!(!FailureCategory::Cancellation.default_severity().is_elevated());
        assert!(FailureCategory::UnexpectedAcceptance.is_security_sensitive());
        assert!(FailureCategory::PrivacyFailure.is_security_sensitive());
        assert!(!FailureCategory::Timeout.is_security_sensitive());
    }

    #[test]
    fn failure_builds_with_classified_context() {
        let failure = Failure::new(
            FailureCategory::AuthorizationFailure,
            LifecycleStage::Execute,
            "unauthorized deposit accepted",
        )
        .with_operation("op-3")
        .with_regression("REG-2026-002")
        .with_detail("actor=unauthorized-user");
        assert_eq!(failure.category, FailureCategory::AuthorizationFailure);
        assert_eq!(failure.stage, LifecycleStage::Execute);
        assert!(failure.severity.is_elevated());
        assert!(failure.is_security_sensitive());
        assert_eq!(failure.operation.as_deref(), Some("op-3"));
    }

    #[test]
    fn failure_serde_round_trip() {
        let f = Failure::new(FailureCategory::StateFailure, LifecycleStage::Assert, "diverged")
            .with_operation("op-1");
        let json = serde_json::to_string(&f).unwrap();
        assert!(json.contains("\"category\":\"STATE_FAILURE\""));
        let back: Failure = serde_json::from_str(&json).unwrap();
        assert_eq!(back, f);
    }

    #[test]
    fn stages_display_stably() {
        assert_eq!(LifecycleStage::Validate.as_str(), "VALIDATE");
        assert_eq!(LifecycleStage::Execute.to_string(), "EXECUTE");
        assert_eq!(LifecycleStage::Cleanup.to_string(), "CLEANUP");
    }
}
