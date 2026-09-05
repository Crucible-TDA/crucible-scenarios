//! The outcome of a scenario run.
//!
//! A [`ScenarioOutcome`] aggregates everything a single run produced: the
//! status, the environment and seed that produced it (reproducibility), the
//! observations and assertion results, and the classified failure when there
//! is one. Full structured *reports* across many outcomes belong to the
//! reporting crate; this is the per-scenario record it will consume.
//!
//! Outcomes serialize with the same structural redaction as observations:
//! private/sensitive values are emitted as markers, never raw.

use serde::{Deserialize, Serialize};

use crate::assertion::AssertionResult;
use crate::environment::Environment;
use crate::failure::Failure;
use crate::observation::ObservationLog;
use crate::scenario_id::ScenarioId;
use crate::seed::Seed;

/// Terminal status of a scenario run.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    /// All expectations and assertions held; the run passed.
    #[default]
    Pass,
    /// A declared expectation/assertion/invariant failed.
    Fail,
    /// The scenario could not run here (capability/environment mismatch) and
    /// was skipped deliberately — never silently mis-executed.
    Skipped,
    /// The scenario declared an expected failure and the system failed
    /// exactly as expected.
    ExpectedFailure,
    /// The run errored before producing a valid result (harness problem).
    Error,
    /// The run exceeded its timeout.
    Timeout,
    /// The run was cancelled.
    Cancelled,
}

impl Status {
    /// Stable machine name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Status::Pass => "PASS",
            Status::Fail => "FAIL",
            Status::Skipped => "SKIPPED",
            Status::ExpectedFailure => "EXPECTED_FAILURE",
            Status::Error => "ERROR",
            Status::Timeout => "TIMEOUT",
            Status::Cancelled => "CANCELLED",
        }
    }

    /// Whether this status represents a run that behaved as the scenario
    /// required (pass, or expected-failure observed).
    pub const fn is_pass(self) -> bool {
        matches!(self, Status::Pass | Status::ExpectedFailure)
    }

    /// Whether this status represents a run that did not behave as required.
    pub const fn is_failure(self) -> bool {
        matches!(
            self,
            Status::Fail | Status::Error | Status::Timeout | Status::Cancelled
        )
    }

    /// Whether the scenario never actually executed.
    pub const fn is_inconclusive(self) -> bool {
        matches!(self, Status::Skipped)
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Status {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.trim().to_ascii_uppercase().as_str() {
            "PASS" => Ok(Status::Pass),
            "FAIL" => Ok(Status::Fail),
            "SKIPPED" => Ok(Status::Skipped),
            "EXPECTED_FAILURE" => Ok(Status::ExpectedFailure),
            "ERROR" => Ok(Status::Error),
            "TIMEOUT" => Ok(Status::Timeout),
            "CANCELLED" => Ok(Status::Cancelled),
            other => Err(format!("unknown status `{other}`")),
        }
    }
}

/// The complete record of one scenario run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScenarioOutcome {
    /// Scenario that ran.
    pub scenario_id: ScenarioId,
    /// Terminal status.
    pub status: Status,
    /// Environment the run executed in.
    pub environment: Environment,
    /// Seed used (always reported, for reproducibility).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<Seed>,
    /// Wall-clock start (epoch milliseconds).
    pub started_at_ms: u64,
    /// Wall-clock finish (epoch milliseconds).
    pub finished_at_ms: u64,
    /// Number of operations executed.
    pub operations_executed: u64,
    /// Observations captured during the run (redacted on serialization).
    #[serde(default)]
    pub observations: ObservationLog,
    /// Per-assertion results.
    #[serde(default)]
    pub assertions: Vec<AssertionResult>,
    /// Number of invariants that held.
    pub invariants_passed: u64,
    /// Number of invariants that failed.
    pub invariants_failed: u64,
    /// Classified failure, when the run failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<Failure>,
}

impl ScenarioOutcome {
    /// Begin an outcome for a scenario starting now.
    pub fn started(scenario_id: ScenarioId, started_at_ms: u64) -> Self {
        ScenarioOutcome {
            scenario_id,
            status: Status::Pass,
            environment: Environment::default(),
            seed: None,
            started_at_ms,
            finished_at_ms: started_at_ms,
            operations_executed: 0,
            observations: ObservationLog::new(),
            assertions: Vec::new(),
            invariants_passed: 0,
            invariants_failed: 0,
            failure: None,
        }
    }

    /// Mark the run finished and set its status.
    pub fn finish(mut self, status: Status, finished_at_ms: u64) -> Self {
        self.status = status;
        self.finished_at_ms = finished_at_ms;
        self
    }

    /// Record the environment this run used.
    pub fn with_environment(mut self, environment: Environment) -> Self {
        self.environment = environment;
        self
    }

    /// Record the seed that produced this run.
    pub fn with_seed(mut self, seed: Seed) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Count an executed operation.
    pub fn op_executed(&mut self) {
        self.operations_executed += 1;
    }

    /// Append an observation.
    pub fn observe(
        &mut self,
        key: impl Into<String>,
        value: impl Into<crate::observation::ObservationValue>,
    ) {
        self.observations
            .push(crate::observation::Observation::public(key, value));
    }

    /// Append a raw observation.
    pub fn push_observation(&mut self, observation: crate::observation::Observation) {
        self.observations.push(observation);
    }

    /// Record an assertion result.
    pub fn push_assertion(&mut self, result: AssertionResult) {
        self.assertions.push(result);
    }

    /// Record invariant results in bulk.
    pub fn set_invariants(&mut self, passed: u64, failed: u64) {
        self.invariants_passed = passed;
        self.invariants_failed = failed;
    }

    /// Attach the classified failure (and set status to [`Status::Fail`]
    /// unless a more specific status is already set).
    pub fn with_failure(mut self, failure: Failure) -> Self {
        if self.status == Status::Pass {
            self.status = Status::Fail;
        }
        self.failure = Some(failure);
        self
    }

    /// Wall-clock duration of the run.
    pub fn duration_ms(&self) -> u64 {
        self.finished_at_ms.saturating_sub(self.started_at_ms)
    }

    /// Whether the run behaved as required.
    pub fn passed(&self) -> bool {
        self.status.is_pass()
    }

    /// Whether the run failed.
    pub fn failed(&self) -> bool {
        self.status.is_failure()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_names_and_semantics() {
        for (status, name) in [
            (Status::Pass, "PASS"),
            (Status::Fail, "FAIL"),
            (Status::Skipped, "SKIPPED"),
            (Status::ExpectedFailure, "EXPECTED_FAILURE"),
            (Status::Error, "ERROR"),
            (Status::Timeout, "TIMEOUT"),
            (Status::Cancelled, "CANCELLED"),
        ] {
            assert_eq!(status.as_str(), name);
            assert_eq!(status.to_string().parse::<Status>().unwrap(), status);
            assert_eq!(
                serde_json::to_string(&status).unwrap(),
                format!("\"{name}\"")
            );
        }
        assert!(Status::Pass.is_pass());
        assert!(Status::ExpectedFailure.is_pass());
        assert!(!Status::ExpectedFailure.is_failure());
        assert!(Status::Timeout.is_failure());
        assert!(Status::Skipped.is_inconclusive());
    }

    #[test]
    fn outcome_tracks_lifecycle_and_failure() {
        let id = ScenarioId::new("CT-XFER-001").unwrap();
        let outcome = ScenarioOutcome::started(id.clone(), 1000)
            .with_environment(Environment::simulator())
            .with_seed(Seed::new(7))
            .finish(Status::Fail, 1560);
        assert_eq!(outcome.duration_ms(), 560);
        assert!(outcome.failed());
        assert_eq!(outcome.seed, Some(Seed::new(7)));
    }

    #[test]
    fn outcome_attaches_failure_and_flips_pass() {
        let id = ScenarioId::new("CT-XFER-001").unwrap();
        let mut outcome =
            ScenarioOutcome::started(id, 0).with_environment(Environment::simulator());
        outcome.op_executed();
        outcome.push_assertion(AssertionResult::failed(
            "a1",
            crate::severity::Severity::Critical,
            "balance diverged",
        ));
        let outcome = outcome.with_failure(Failure::new(
            crate::failure::FailureCategory::UnexpectedAcceptance,
            crate::failure::LifecycleStage::Execute,
            "invalid proof accepted",
        ));
        assert_eq!(outcome.status, Status::Fail);
        assert_eq!(outcome.operations_executed, 1);
        assert_eq!(outcome.assertions.len(), 1);
        assert!(outcome.failure.is_some());
    }

    #[test]
    fn outcome_serializes_without_leaking_private_values() {
        let id = ScenarioId::new("CT-PRIV-001").unwrap();
        let mut outcome =
            ScenarioOutcome::started(id, 0).with_environment(Environment::simulator());
        outcome.push_observation(crate::observation::Observation::private(
            "op-1.amount",
            55_000,
        ));
        let json = serde_json::to_string(&outcome).unwrap();
        assert!(
            !json.contains("55000"),
            "private amount leaked into outcome JSON"
        );
        assert!(json.contains("[REDACTED]"));
    }
}
