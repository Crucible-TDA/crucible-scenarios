//! The runtime handle a scenario executes through.
//!
//! [`ScenarioContext`] is the *only* channel between a scenario and the
//! outside world. It carries the stable integration contracts — simulation,
//! proof generation, verification, Soroban invocation, fixtures, the clock,
//! and randomness — as trait objects, so a scenario definition drives whatever
//! adapter is plugged in without ever reaching into the internals of
//! crucible-simulator, crucible-prover, or a Soroban deployment.
//!
//! The traits here are the repository's public interface *contracts*: the
//! adapter crates implement them for real systems, and this crate's unit
//! tests plus the runner's mock environment implement them with explicit test
//! doubles. Contracts are deliberately expressed in this repository's own
//! vocabulary (operations, observations, opaque proof/state references) so
//! that no adapter dependency leaks into scenario definitions.

use serde::Serialize;

use crate::actors::ActorSet;
use crate::capabilities::{Capabilities, Capability};
use crate::environment::Environment;
use crate::errors::{Error, Result};
use crate::metadata::ScenarioMetadata;
use crate::operation::Operation;
use crate::scenario_id::ScenarioId;
use crate::seed::{DeterministicRng, Seed};

/// A wall-clock source. Deterministic scenarios use a [`FixedClock`]; real
/// runs use [`SystemClock`].
pub trait Clock: Send + Sync {
    /// Current time in epoch milliseconds.
    fn now_ms(&self) -> u64;
}

/// The system clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

static SYSTEM_CLOCK: SystemClock = SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

/// A fixed clock for deterministic runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedClock(pub u64);

impl Clock for FixedClock {
    fn now_ms(&self) -> u64 {
        self.0
    }
}

/// Report of one operation executed against a simulation/Soroban surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExecutionReport {
    /// Whether the surface accepted the operation.
    pub accepted: bool,
    /// Stable status code (`succeeded`, `rejected`, …).
    pub status_code: String,
    /// State digest the surface produced, when exposed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_digest: Option<String>,
    /// Public event codes emitted.
    #[serde(default)]
    pub event_codes: Vec<String>,
    /// Rejection/error code, when the operation was refused.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
}

/// The simulation contract that crucible-simulator adapters implement.
///
/// This crate intentionally does not interpret the report: judging whether
/// the reported state is *correct* is the scenario's job through its
/// expectations and invariants (the anti-circular-testing rule), so the
/// contract stays a transport, not an oracle.
pub trait SimulatorService: Send + Sync {
    /// Human-readable adapter name for reporting.
    fn name(&self) -> &str;

    /// Execute one scenario operation against the simulator.
    fn execute(&self, operation: &Operation) -> Result<ExecutionReport>;
}

/// Proof-generation request: the operation and its *public* inputs.
///
/// Witnesses never cross this interface as plain values; a scenario refers to
/// a witness by reference through the fixture/context plumbing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofRequest<'a> {
    /// Operation the proof is for.
    pub operation: &'a Operation,
    /// Public inputs (name → value). Only public protocol values belong here.
    pub public_inputs: Vec<(String, String)>,
}

/// Report of a proof-generation attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProofReport {
    /// Opaque proof identifier for later verification.
    pub proof_id: String,
    /// Digest of the verification key used, when exposed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_key_digest: Option<String>,
}

/// The proof-generation contract implemented by crucible-prover adapters.
pub trait ProofProviderService: Send + Sync {
    /// Human-readable adapter name.
    fn name(&self) -> &str;

    /// Generate a proof for the request, returning an opaque handle.
    fn generate(&self, request: ProofRequest<'_>) -> Result<ProofReport>;
}

/// Verification request: which proof, against which public inputs and state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationRequest<'a> {
    /// Proof to check.
    pub proof_id: &'a str,
    /// Public inputs the proof must bind to.
    pub public_inputs: Vec<(String, String)>,
    /// State digest the proof must bind to, when bound to state.
    pub state_digest: Option<&'a str>,
}

/// Result of one verification attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerificationReport {
    /// Whether the proof verified.
    pub valid: bool,
    /// Why verification failed, when it did.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// The verification contract implemented by verifier adapters.
pub trait VerifierService: Send + Sync {
    /// Human-readable adapter name.
    fn name(&self) -> &str;

    /// Verify a proof against bound public inputs/state.
    fn verify(&self, request: VerificationRequest<'_>) -> Result<VerificationReport>;
}

/// The Soroban contract-invocation contract. It only *invokes* deployed or
/// local Confidential Token contracts through stable interfaces; it never
/// implements them.
pub trait SorobanService: Send + Sync {
    /// Human-readable adapter name.
    fn name(&self) -> &str;

    /// Invoke a contract function with JSON-ish public arguments.
    fn invoke(
        &self,
        contract: &str,
        function: &str,
        public_args: Vec<(String, String)>,
    ) -> Result<ExecutionReport>;
}

/// The fixture contract: load a fixture by category and key as a JSON value.
///
/// Fixture *types* (accounts, tokens, proofs, …) live in the fixtures crate;
/// the context only needs a neutral load channel.
pub trait FixtureProvider: Send + Sync {
    /// Load a fixture.
    fn load(&self, category: &str, key: &str) -> Result<serde_json::Value>;
}

/// A live, append-only record of public event codes observed during a run.
///
/// Thread-safe so parallel execution can share one sink; only *public* codes
/// are recorded here, so sharing cannot leak private data.
#[derive(Debug, Default)]
pub struct EventLog {
    inner: std::sync::Mutex<Vec<String>>,
}

impl EventLog {
    /// A new empty event log.
    pub fn new() -> Self {
        EventLog {
            inner: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Record a public event code.
    pub fn record(&self, code: impl Into<String>) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.push(code.into());
        }
    }

    /// Public event codes in emission order.
    pub fn codes(&self) -> Vec<String> {
        self.inner.lock().map(|inner| inner.clone()).unwrap_or_default()
    }

    /// Whether any event with the code was recorded.
    pub fn contains(&self, code: &str) -> bool {
        self.inner
            .lock()
            .map(|inner| inner.iter().any(|c| c == code))
            .unwrap_or(false)
    }

    /// Number of recorded events.
    pub fn len(&self) -> usize {
        self.inner.lock().map(|inner| inner.len()).unwrap_or(0)
    }

    /// Whether nothing was recorded.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Everything a scenario needs to execute, without implementation internals.
#[derive(Default)]
pub struct ScenarioContext {
    scenario_id: Option<ScenarioId>,
    metadata: Option<ScenarioMetadata>,
    environment: Option<Environment>,
    seed: Option<Seed>,
    /// Capabilities *offered* by the plugged-in services and environment.
    offered: Capabilities,
    actors: ActorSet,
    clock: Option<Box<dyn Clock>>,
    simulator: Option<Box<dyn SimulatorService>>,
    prover: Option<Box<dyn ProofProviderService>>,
    verifier: Option<Box<dyn VerifierService>>,
    soroban: Option<Box<dyn SorobanService>>,
    fixtures: Option<Box<dyn FixtureProvider>>,
    events: EventLog,
}

impl std::fmt::Debug for ScenarioContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScenarioContext")
            .field("scenario_id", &self.scenario_id)
            .field("environment", &self.environment)
            .field("seed", &self.seed)
            .field("offered", &self.offered)
            .field("services", &self.service_names())
            .finish_non_exhaustive()
    }
}

impl ScenarioContext {
    /// Begin an empty context for a scenario.
    pub fn for_scenario(
        metadata: ScenarioMetadata,
        environment: Environment,
        seed: Option<Seed>,
    ) -> Self {
        let mut context = ScenarioContext {
            scenario_id: Some(metadata.id.clone()),
            metadata: Some(metadata),
            environment: Some(environment),
            seed,
            ..ScenarioContext::default()
        };
        context.offered = Capabilities::implied_by(context.environment().kind);
        context
    }

    /// The scenario id this context was built for.
    pub fn scenario_id(&self) -> &ScenarioId {
        self.scenario_id
            .as_ref()
            .expect("context built for a scenario always has an id")
    }

    /// The scenario metadata.
    pub fn metadata(&self) -> &ScenarioMetadata {
        self.metadata.as_ref().expect("context always has metadata")
    }

    /// The environment this context drives.
    pub fn environment(&self) -> &Environment {
        self.environment.as_ref().expect("context always has an environment")
    }

    /// The deterministic seed, when the scenario declared one.
    pub fn seed(&self) -> Option<Seed> {
        self.seed
    }

    /// Capabilities this context can actually offer.
    pub fn offered_capabilities(&self) -> &Capabilities {
        &self.offered
    }

    /// Check that the context can cover the required capabilities.
    pub fn ensure_capabilities(&self, required: &Capabilities) -> Result<()> {
        self.offered.ensure_covers(required)
    }

    /// The scenario's declared actors.
    pub fn actors(&self) -> &ActorSet {
        &self.actors
    }

    /// Declare the actors participating (must match the scenario).
    pub fn with_actors(mut self, actors: ActorSet) -> Self {
        self.actors = actors;
        self
    }

    /// Plug in a clock.
    pub fn with_clock(mut self, clock: impl Clock + 'static) -> Self {
        self.clock = Some(Box::new(clock));
        self
    }

    /// Plug in a simulator service (implies the `simulation` capability).
    pub fn with_simulator(mut self, service: impl SimulatorService + 'static) -> Self {
        self.offered.insert(Capability::Simulation);
        self.simulator = Some(Box::new(service));
        self
    }

    /// Plug in a proof provider (implies `proof-provider`).
    pub fn with_prover(mut self, service: impl ProofProviderService + 'static) -> Self {
        self.offered.insert(Capability::ProofProvider);
        self.prover = Some(Box::new(service));
        self
    }

    /// Plug in a verifier (implies `verifier`).
    pub fn with_verifier(mut self, service: impl VerifierService + 'static) -> Self {
        self.offered.insert(Capability::Verifier);
        self.verifier = Some(Box::new(service));
        self
    }

    /// Plug in a Soroban service (implies `soroban-adapter`).
    pub fn with_soroban(mut self, service: impl SorobanService + 'static) -> Self {
        self.offered.insert(Capability::SorobanAdapter);
        self.soroban = Some(Box::new(service));
        self
    }

    /// Plug in a fixture provider.
    pub fn with_fixtures(mut self, provider: impl FixtureProvider + 'static) -> Self {
        self.fixtures = Some(Box::new(provider));
        self
    }

    /// Clock access.
    pub fn clock(&self) -> &dyn Clock {
        self.clock.as_deref().unwrap_or(&SYSTEM_CLOCK)
    }

    /// Simulator access.
    pub fn simulator(&self) -> Result<&dyn SimulatorService> {
        self.simulator
            .as_deref()
            .ok_or_else(|| Error::UnavailableCapability(Capability::Simulation.as_str().into()))
    }

    /// Proof-provider access.
    pub fn prover(&self) -> Result<&dyn ProofProviderService> {
        self.prover
            .as_deref()
            .ok_or_else(|| Error::UnavailableCapability(Capability::ProofProvider.as_str().into()))
    }

    /// Verifier access.
    pub fn verifier(&self) -> Result<&dyn VerifierService> {
        self.verifier
            .as_deref()
            .ok_or_else(|| Error::UnavailableCapability(Capability::Verifier.as_str().into()))
    }

    /// Soroban access.
    pub fn soroban(&self) -> Result<&dyn SorobanService> {
        self.soroban.as_deref().ok_or_else(|| {
            Error::UnavailableCapability(Capability::SorobanAdapter.as_str().into())
        })
    }

    /// Fixture access.
    pub fn fixtures(&self) -> Result<&dyn FixtureProvider> {
        self.fixtures
            .as_deref()
            .ok_or_else(|| Error::Internal("no fixture provider configured".into()))
    }

    /// The public event log of this run.
    pub fn events(&self) -> &EventLog {
        &self.events
    }

    /// A deterministic randomness stream derived from the scenario seed, or a
    /// fixed default when the scenario declared none (still deterministic).
    pub fn rng(&self) -> DeterministicRng {
        DeterministicRng::new(self.seed.unwrap_or(Seed::ZERO))
    }

    /// A child seed for a named sub-stream (e.g. per fixture category), so
    /// every randomized consumer gets isolated, reproducible randomness.
    pub fn seed_for(&self, label: &str) -> Seed {
        let base = self.seed.unwrap_or(Seed::ZERO);
        // Fold the label into a stable index via a deterministic hash.
        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        for byte in label.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x1000_0000_01b3);
        }
        base.child(hash)
    }

    /// Names of the currently plugged-in services (for Debug/logs).
    pub fn service_names(&self) -> Vec<&'static str> {
        let mut names = Vec::new();
        if self.simulator.is_some() {
            names.push("simulator");
        }
        if self.prover.is_some() {
            names.push("prover");
        }
        if self.verifier.is_some() {
            names.push("verifier");
        }
        if self.soroban.is_some() {
            names.push("soroban");
        }
        if self.fixtures.is_some() {
            names.push("fixtures");
        }
        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct RecordingSimulator {
        events: std::sync::Arc<EventLog>,
    }

    impl SimulatorService for RecordingSimulator {
        fn name(&self) -> &str {
            "recording"
        }

        fn execute(&self, _operation: &Operation) -> Result<ExecutionReport> {
            self.events.record("op-executed");
            Ok(ExecutionReport {
                accepted: true,
                status_code: "succeeded".to_string(),
                state_digest: Some("state-s3".to_string()),
                event_codes: vec!["ct_xfer".to_string()],
                error_code: None,
            })
        }
    }

    fn metadata() -> ScenarioMetadata {
        ScenarioMetadata::new(
            ScenarioId::new("CT-XFER-001").unwrap(),
            "transfer",
            "desc",
            crate::metadata::Category::HappyPath,
        )
        .unwrap()
    }

    #[test]
    fn context_starts_with_environment_implied_capabilities() {
        let context = ScenarioContext::for_scenario(
            metadata(),
            Environment::simulator(),
            Some(Seed::new(1)),
        );
        assert!(context.offered_capabilities().has(Capability::Simulation));
        assert!(!context.offered_capabilities().has(Capability::Testnet));
        let required = Capabilities::of([Capability::Simulation]);
        assert!(context.ensure_capabilities(&required).is_ok());
        let testnet = Capabilities::of([Capability::Testnet]);
        assert!(context.ensure_capabilities(&testnet).is_err());
    }

    #[test]
    fn services_offer_their_capabilities() {
        let events = EventLog::new();
        let events_ref = std::sync::Arc::new(events);
        let context = ScenarioContext::for_scenario(
            metadata(),
            Environment::simulator(),
            Some(Seed::new(1)),
        )
        .with_simulator(RecordingSimulator {
            events: std::sync::Arc::clone(&events_ref),
        })
        .with_prover(AlwaysOkProver)
        .with_verifier(AlwaysTrueVerifier);
        assert!(context.offered_capabilities().has(Capability::ProofProvider));
        assert!(context.offered_capabilities().has(Capability::Verifier));
        assert!(context.prover().is_ok());
        assert!(context.soroban().is_err());
        assert_eq!(context.service_names(), vec!["simulator", "prover", "verifier"]);
    }

    struct AlwaysOkProver;
    impl ProofProviderService for AlwaysOkProver {
        fn name(&self) -> &str {
            "always-ok"
        }

        fn generate(&self, _request: ProofRequest<'_>) -> Result<ProofReport> {
            Ok(ProofReport {
                proof_id: "proof-1".to_string(),
                verification_key_digest: None,
            })
        }
    }

    struct AlwaysTrueVerifier;
    impl VerifierService for AlwaysTrueVerifier {
        fn name(&self) -> &str {
            "always-true"
        }

        fn verify(&self, _request: VerificationRequest<'_>) -> Result<VerificationReport> {
            Ok(VerificationReport {
                valid: true,
                error: None,
            })
        }
    }

    #[test]
    fn clock_and_fixed_clock() {
        let context = ScenarioContext::for_scenario(metadata(), Environment::simulator(), None)
            .with_clock(FixedClock(1_700_000_000_000));
        assert_eq!(context.clock().now_ms(), 1_700_000_000_000);
        assert!(SystemClock.now_ms() > 0);
    }

    #[test]
    fn sub_seeds_are_stable_per_label() {
        let a = ScenarioContext::for_scenario(metadata(), Environment::simulator(), Some(Seed::new(9)));
        let b = ScenarioContext::for_scenario(metadata(), Environment::simulator(), Some(Seed::new(9)));
        assert_eq!(a.seed_for("balances"), b.seed_for("balances"));
        assert_ne!(a.seed_for("balances"), a.seed_for("proofs"));
        // Different parent seeds give different sub-streams for the same label.
        let c = ScenarioContext::for_scenario(metadata(), Environment::simulator(), Some(Seed::new(10)));
        assert_ne!(a.seed_for("balances"), c.seed_for("balances"));
    }

    #[test]
    fn event_log_records_public_codes() {
        let log = EventLog::new();
        log.record("ct_xfer");
        log.record("ct_xfer");
        assert_eq!(log.len(), 2);
        assert!(log.contains("ct_xfer"));
        assert!(!log.contains("ct_deposit"));
    }
}
