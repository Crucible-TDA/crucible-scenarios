//! `scenario-core` — the domain model of the crucible-scenarios STRESS-TEST
//! polyrepo.
//!
//! This crate **defines** the vocabulary of scenario-based validation. It
//! never executes scenarios itself: execution, discovery, fixtures,
//! assertions, adapters, and reporting live in sibling crates that consume
//! these types.
//!
//! Owned concepts:
//!
//! * `Scenario` / `ScenarioId` / `ScenarioMetadata` — stable, versioned
//!   scenario definitions.
//! * `Actor` / `Environment` / `Capabilities` — who runs, where, and what the
//!   run may touch.
//! * `Operation` / `Expectation` / `Observation` / `Assertion` — what a
//!   scenario does, expects, sees, and checks.
//! * `ScenarioOutcome` / `Failure` / `FailureCategory` / `Severity` — how a
//!   run is classified.
//! * `Seed` — deterministic replay and generation.
//! * `ScenarioContext` — the runtime handle that carries the stable
//!   integration interfaces (simulation, proving, verification, Soroban,
//!   fixtures, events, clock, randomness) without exposing implementation
//!   internals of the other Crucible polyrepos.
//!
//! ## Repository boundaries
//!
//! `scenario-core` stays free of protocol implementation: no token
//! accounting, no cryptographic primitives, no circuit logic. Confidential
//! amounts and witnesses are represented as redaction-aware values so that
//! private information cannot leak through `Debug`, logs, or serialized
//! results.
//!
//! Modules are attached one improvement at a time so the crate always
//! compiles.

pub mod actors;
pub mod capabilities;
pub mod environment;
pub mod errors;
pub mod expectation;
pub mod metadata;
pub mod observation;
pub mod operation;
pub mod scenario_id;
pub mod seed;
pub mod severity;
pub mod tags;

pub use actors::{Actor, ActorId, ActorSet, Role};
pub use capabilities::{Capabilities, Capability};
pub use environment::{Environment, EnvironmentKind};
pub use errors::{Error, Result};
pub use expectation::{Expectation, ExpectationKind};
pub use metadata::{Category, ScenarioMetadata};
pub use observation::{Observation, ObservationLog, ObservationValue, Visibility, REDACTED_MARKER};
pub use operation::{Amount, ConfidentialAmount, Operation, OperationId, OperationKind, TokenId};
pub use scenario_id::ScenarioId;
pub use seed::{DeterministicRng, Seed};
pub use severity::Severity;
pub use tags::Tags;
