# Scenario model

The scenario model lives in `crates/scenario-core`. It **defines** the
vocabulary of scenario-based validation and never executes anything.

## Core types

| Type | Meaning |
| --- | --- |
| `Scenario` | A complete definition: metadata, environment, capabilities, actors, operations, expectations, assertions, invariants, seed, timeout, declared outcome. Validated at build time. |
| `ScenarioId` | Permanent, grammar-validated identifier (`CT-XFER-001`, `CT-XFER-NEG-001`, `CT-PROOF-REPLAY-001`, `REG-2026-001`). |
| `ScenarioMetadata` | Name, description, `Category`, `Tags`, pinned protocol/circuit/prover/simulator versions, references. |
| `Actor` / `ActorId` / `Role` | Public synthetic identities (alice, bob, issuer, auditor, unauthorized…) with declared roles. Actors never carry credentials. |
| `Environment` / `EnvironmentKind` | Where a run executes: mock, simulator, prover, soroban, testnet, end-to-end. Testnet is isolated. |
| `Capability` / `Capabilities` | What a scenario requires vs. what a context offers (simulation, proof-provider, verifier, soroban-adapter, testnet, event-observation, snapshots, concurrency, replay-protection, deterministic-clock). |
| `Operation` / `OperationKind` | Typed workflow steps: register, deposit, merge, confidential transfer, withdraw. Intents, not implementations. |
| `Expectation` | Declarative claims: succeeds, rejected (with optional reason), replay-rejected, invariant-holds, not-disclosed. |
| `AssertionSpec` / `AssertionResult` | Declared checks and their per-run outcomes. |
| `Observation` / `ObservationLog` | What the run saw, classified public/private/sensitive/internal. |
| `ScenarioOutcome` / `Status` | The per-run record: status, environment, seed, timings, observations, assertion results, invariants, failure. |
| `Failure` / `FailureCategory` / `LifecycleStage` | Classified findings with category, stage, severity. |
| `Severity` | info/low/medium/high/critical with elevated markers for security findings. |
| `Seed` | Deterministic 64-bit seed; children per consumer; replay key. |
| `ScenarioContext` | The runtime handle: clock, rng, actors, event sink, and the stable service contracts (simulator, prover, verifier, soroban, fixtures). |

## Redaction by construction

`ConfidentialAmount` renders `[redacted]` in `Debug`/`Display`.
`Observation` values classified private/sensitive/internal serialize only as
`[REDACTED]`; `ScenarioOutcome` inherits that. Raw private values exist only
in memory for trusted executor/assertion code. Scenario **definition** files
may carry expected private values; runtime results must not.

## Scenario IDs and stable references

Scenario IDs are permanent and grammar-checked (uppercase A–Z, 0–9, single
dashes, ≤ 64 chars). Actor/token/operation identifiers are lowercase slugs.
Every expectation/assertion that names an operation is validated against the
scenario's operation list at build time, so dangling references cannot be
silently ignored.

## Whole-scenario semantics

`ScenarioBuilder::declared_outcome` states whether the scenario as a whole
succeeds or is *declared* to end in a specific failure category. A declared
failure passes only if the system fails exactly that way (`Status::ExpectedFailure`
counts as a pass). Rejections are expected first-class behavior in negative
scenarios: an expected rejection is a pass; only unexpected acceptance or
unexpected rejection is a defect.

## Capabilities and environments

A scenario's environment implies base capabilities (`simulator` →
simulation/snapshots/event-observation, `prover` → proof-provider/verifier,
…). `ScenarioContext` derives its *offered* capabilities from the environment
plus whichever services are actually plugged in, and refuses to run a
scenario whose requirements it cannot cover (`UnavailableCapability`). A
testnet scenario can therefore never silently run in ordinary CI.
