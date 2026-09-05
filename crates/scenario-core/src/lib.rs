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

pub mod errors;
pub mod seed;

pub use errors::{Error, Result};
pub use seed::{DeterministicRng, Seed};
