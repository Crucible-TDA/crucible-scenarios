//! The scenario aggregate.
//!
//! A [`Scenario`] is a complete, self-describing definition: stable metadata,
//! the environment it targets, the capabilities it requires, the actors that
//! take part, the ordered operations to perform, the expectations and
//! assertions that judge the run, the invariants that must hold across it, a
//! deterministic seed, and a timeout.
//!
//! Building a [`Scenario`] **validates** it up front: duplicate operation
//! ids, expectations that reference operations which do not exist, and
//! malformed timeouts are definition errors that must surface before any
//! execution starts — never as confusing mid-run failures. A scenario that
//! cannot be executed honestly (capability/environment mismatch) is skipped
//! at run time, never silently mis-executed.

use serde::{Deserialize, Serialize};

use crate::actors::{Actor, ActorSet, ActorId};
use crate::assertion::AssertionSpec;
use crate::capabilities::{Capabilities, Capability};
use crate::environment::Environment;
use crate::errors::{Error, Result};
use crate::expectation::Expectation;
use crate::failure::FailureCategory;
use crate::metadata::ScenarioMetadata;
use crate::operation::{Operation, OperationId};
use crate::scenario_id::ScenarioId;
use crate::seed::Seed;
use crate::tags::Tags;

/// What the scenario as a whole declares will happen.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeclaredOutcome {
    /// The scenario expects its operations and assertions to succeed.
    #[default]
    Succeeds,
    /// The scenario is *declared* to end in a failure of the given category;
    /// it passes only if the system fails exactly that way.
    Fails(FailureCategory),
}

/// A complete scenario definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scenario {
    /// Identity and description.
    pub metadata: ScenarioMetadata,
    /// Target environment.
    pub environment: Environment,
    /// Capabilities this scenario requires from the context.
    #[serde(default)]
    pub required_capabilities: Capabilities,
    /// The actors taking part.
    #[serde(default)]
    pub actors: ActorSet,
    /// Ordered operations to perform.
    #[serde(default)]
    pub operations: Vec<Operation>,
    /// Expected behavior.
    #[serde(default)]
    pub expectations: Vec<Expectation>,
    /// Declarative checks.
    #[serde(default)]
    pub assertions: Vec<AssertionSpec>,
    /// Cross-operation invariants that must hold (registry keys).
    #[serde(default)]
    pub invariant_ids: Vec<String>,
    /// Deterministic seed when the scenario randomizes anything.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<Seed>,
    /// Whole-scenario expected outcome semantics.
    #[serde(default)]
    pub declared_outcome: DeclaredOutcome,
    /// Timeout in milliseconds (positive when set).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

impl Scenario {
    /// Start building a scenario from its metadata.
    pub fn builder(metadata: ScenarioMetadata) -> ScenarioBuilder {
        ScenarioBuilder::new(metadata)
    }

    /// Stable scenario id.
    pub fn id(&self) -> &ScenarioId {
        &self.metadata.id
    }

    /// The scenario's tags.
    pub fn tags(&self) -> &Tags {
        &self.metadata.tags
    }

    /// Look up an operation by id.
    pub fn operation(&self, id: &OperationId) -> Option<&Operation> {
        self.operations.iter().find(|op| &op.id == id)
    }

    /// Look up an actor by id.
    pub fn actor(&self, id: &ActorId) -> Option<&Actor> {
        self.actors.get(id)
    }

    /// Whether this scenario targets an isolated environment (testnet).
    pub fn is_isolated(&self) -> bool {
        self.environment.is_isolated()
    }

    /// Whether the declared whole-scenario outcome is failure.
    pub fn declared_failure(&self) -> Option<FailureCategory> {
        match self.declared_outcome {
            DeclaredOutcome::Succeeds => None,
            DeclaredOutcome::Fails(category) => Some(category),
        }
    }
}

/// Builder that assembles and validates a [`Scenario`].
#[derive(Debug, Clone, Default)]
pub struct ScenarioBuilder {
    metadata: Option<ScenarioMetadata>,
    environment: Environment,
    required_capabilities: Capabilities,
    actors: ActorSet,
    operations: Vec<Operation>,
    expectations: Vec<Expectation>,
    assertions: Vec<AssertionSpec>,
    invariant_ids: Vec<String>,
    seed: Option<Seed>,
    declared_outcome: DeclaredOutcome,
    timeout_ms: Option<u64>,
}

impl ScenarioBuilder {
    /// Begin with mandatory metadata.
    pub fn new(metadata: ScenarioMetadata) -> Self {
        ScenarioBuilder {
            metadata: Some(metadata),
            ..ScenarioBuilder::default()
        }
    }

    /// Set the target environment (also implies default capabilities; see
    /// [`ScenarioBuilder::require`] to extend them).
    pub fn environment(mut self, environment: Environment) -> Self {
        let implied = Capabilities::implied_by(environment.kind);
        self.environment = environment;
        for cap in implied.iter() {
            self.required_capabilities.insert(cap);
        }
        self
    }

    /// Require an additional capability.
    pub fn require(mut self, capability: Capability) -> Self {
        self.required_capabilities.insert(capability);
        self
    }

    /// Declare the actors of the scenario.
    pub fn actors(mut self, actors: ActorSet) -> Self {
        self.actors = actors;
        self
    }

    /// Add an actor.
    pub fn add_actor(mut self, actor: Actor) -> Result<Self> {
        self.actors.register(actor)?;
        Ok(self)
    }

    /// Add an operation.
    pub fn add_operation(mut self, operation: Operation) -> Self {
        self.operations.push(operation);
        self
    }

    /// Add an expectation.
    pub fn add_expectation(mut self, expectation: Expectation) -> Self {
        self.expectations.push(expectation);
        self
    }

    /// Add an assertion.
    pub fn add_assertion(mut self, assertion: AssertionSpec) -> Self {
        self.assertions.push(assertion);
        self
    }

    /// Require a cross-operation invariant by registry key.
    pub fn add_invariant(mut self, key: impl Into<String>) -> Self {
        self.invariant_ids.push(key.into());
        self
    }

    /// Pin a deterministic seed.
    pub fn seed(mut self, seed: Seed) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Declare whole-scenario failure semantics.
    pub fn declared_outcome(mut self, outcome: DeclaredOutcome) -> Self {
        self.declared_outcome = outcome;
        self
    }

    /// Set the timeout in milliseconds.
    pub fn timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    /// Validate and produce the scenario.
    pub fn build(self) -> Result<Scenario> {
        let metadata = self
            .metadata
            .ok_or_else(|| Error::InvalidMetadata("scenario metadata is required".into()))?;

        // Operation ids must be unique: an ambiguous sequence cannot execute.
        let mut seen = std::collections::BTreeSet::new();
        for op in &self.operations {
            if !seen.insert(op.id.clone()) {
                return Err(Error::DuplicateId(format!("operation `{}`", op.id)));
            }
        }

        // Every expectation must reference an operation that actually exists;
        // dangling expectations would silently never be checked.
        for expectation in &self.expectations {
            if let Some(operation_id) = expectation.referenced_operation() {
                if !seen.contains(operation_id) {
                    return Err(Error::UnknownReference(
                        "expectation",
                        operation_id.to_string(),
                    ));
                }
            }
        }

        // A timeout, when present, must be positive.
        if let Some(timeout_ms) = self.timeout_ms {
            if timeout_ms == 0 {
                return Err(Error::InvalidTimeout(
                    "timeout must be a positive number of milliseconds".into(),
                ));
            }
        }

        // Invariant keys must be non-empty when declared.
        if self.invariant_ids.iter().any(|k| k.trim().is_empty()) {
            return Err(Error::UnknownReference(
                "invariant",
                "<empty>".into(),
            ));
        }

        Ok(Scenario {
            metadata,
            environment: self.environment,
            required_capabilities: self.required_capabilities,
            actors: self.actors,
            operations: self.operations,
            expectations: self.expectations,
            assertions: self.assertions,
            invariant_ids: self.invariant_ids,
            seed: self.seed,
            declared_outcome: self.declared_outcome,
            timeout_ms: self.timeout_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actors::Role;
    use crate::operation::{Amount, ConfidentialAmount, OperationKind, TokenId};

    fn xfer_metadata() -> ScenarioMetadata {
        ScenarioMetadata::new(
            ScenarioId::new("CT-XFER-001").unwrap(),
            "confidential transfer, sufficient balance",
            "alice transfers 30 to bob with a valid proof",
            crate::metadata::Category::HappyPath,
        )
        .unwrap()
        .with_tags(Tags::of([crate::tags::standard::TRANSFER]))
    }

    fn actors() -> ActorSet {
        let mut set = ActorSet::new();
        set.register(Actor::new(ActorId::new("alice").unwrap(), Role::User)).unwrap();
        set.register(Actor::new(ActorId::new("bob").unwrap(), Role::User)).unwrap();
        set
    }

    fn ops() -> Vec<Operation> {
        vec![
            Operation::new(
                OperationId::new("op-1").unwrap(),
                ActorId::new("alice").unwrap(),
                OperationKind::Deposit {
                    token: TokenId::new("ct-usdc").unwrap(),
                    recipient: ActorId::new("alice").unwrap(),
                    amount: Amount::new(100),
                },
            ),
            Operation::new(
                OperationId::new("op-2").unwrap(),
                ActorId::new("alice").unwrap(),
                OperationKind::Transfer {
                    token: TokenId::new("ct-usdc").unwrap(),
                    sender: ActorId::new("alice").unwrap(),
                    recipient: ActorId::new("bob").unwrap(),
                    amount: ConfidentialAmount::new(30),
                },
            ),
        ]
    }

    fn base_builder() -> ScenarioBuilder {
        let mut builder = ScenarioBuilder::new(xfer_metadata())
            .environment(Environment::simulator())
            .actors(actors())
            .seed(Seed::new(42))
            .timeout_ms(5_000);
        for op in ops() {
            builder = builder.add_operation(op);
        }
        builder
    }

    #[test]
    fn scenario_builds_and_exposes_helpers() {
        let scenario = base_builder().build().unwrap();
        assert_eq!(scenario.id().as_str(), "CT-XFER-001");
        assert_eq!(scenario.operations.len(), 2);
        assert!(scenario.tags().contains("transfer"));
        assert!(scenario
            .operation(&OperationId::new("op-2").unwrap())
            .is_some());
        assert!(!scenario.is_isolated());
        assert!(scenario.required_capabilities.has(Capability::Simulation));
    }

    #[test]
    fn duplicate_operation_ids_are_rejected() {
        let op = Operation::new(
            OperationId::new("op-1").unwrap(),
            ActorId::new("alice").unwrap(),
            OperationKind::Register {
                account: ActorId::new("alice").unwrap(),
            },
        );
        let scenario = base_builder()
            .add_operation(op)
            .build();
        assert!(matches!(scenario, Err(Error::DuplicateId(_))));
    }

    #[test]
    fn dangling_expectation_references_are_rejected() {
        let expectation = Expectation::succeeds(
            "e1",
            "ghost operation",
            OperationId::new("op-99").unwrap(),
        );
        let scenario = base_builder().add_expectation(expectation).build();
        assert!(matches!(scenario, Err(Error::UnknownReference("expectation", _))));
    }

    #[test]
    fn zero_timeout_is_rejected() {
        assert!(matches!(
            base_builder().timeout_ms(0).build(),
            Err(Error::InvalidTimeout(_))
        ));
    }

    #[test]
    fn declared_failure_semantics_round_trip() {
        let scenario = base_builder()
            .declared_outcome(DeclaredOutcome::Fails(FailureCategory::AuthorizationFailure))
            .build()
            .unwrap();
        assert_eq!(
            scenario.declared_failure(),
            Some(FailureCategory::AuthorizationFailure)
        );
    }

    #[test]
    fn scenario_serde_round_trip() {
        let scenario = base_builder().build().unwrap();
        let json = serde_json::to_string(&scenario).unwrap();
        let back: Scenario = serde_json::from_str(&json).unwrap();
        assert_eq!(back, scenario);
        assert!(json.contains("\"id\":\"CT-XFER-001\""));
    }
}
