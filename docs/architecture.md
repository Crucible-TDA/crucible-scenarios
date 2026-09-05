# Architecture

`crucible-scenarios` is the **STRESS-TEST** polyrepo of the Crucible hybrid
system: a scenario orchestration and validation layer for the Crucible
Confidential Token stack. It never defines the system — it tests it.

## The three polyrepos

| Polyrepo | Layer | Owns |
| --- | --- | --- |
| `crucible-simulator` | SIMULATE | deterministic simulation and state execution |
| `crucible-prover` | PROVE | circuits, witnesses, proving, verification, proof artifacts, proof interfaces |
| `crucible-scenarios` | STRESS-TEST | scenario definitions/execution, orchestration, fixtures, test vectors, assertions, invariants, failure classification, negative/adversarial/privacy/conformance/regression/compatibility suites, stress/performance/fuzzing orchestration, reports, reproducible replay |

## What belongs here — and what does not

**This repository owns:** scenario definitions and execution; test
orchestration; deterministic fixtures and versioned test vectors; expected
outcomes and reusable assertions; cross-operation invariants; failure
classification; regression cases; negative, adversarial, and privacy testing;
compatibility and conformance checking; concurrency, stress, performance, and
fuzzing orchestration; structured reporting; reproducibility and
deterministic replay; integration-test coordination.

**It does NOT own:** the token contract implementation; Confidential Token
cryptographic implementation; proof/circuit generation; wallet or production
account management; compliance/sanctions policy; audit dashboards; production
transaction orchestration; or any generic blockchain framework functionality.
It must never re-implement simulator or prover semantics, never depend on
their private internals, never modify production contracts from tests, and
never treat mocked proofs as cryptographically valid.

## Data flow

```
 Scenario
    │  (definition: metadata, actors, operations, expectations)
    ▼
 ScenarioContext ──── ScenarioRunner ──── SimulatorService (simulator adapter)
    │                                         │
    │      ProofProviderService (prover       │  operation execution
    │      adapter)  ── witness ──► proof     ▼
    │      VerifierService                    state transition + events
    ▼                                         │
 Observations (classified public/private) ◄───┘
    │
    ▼
 Assertions ──► Invariants ──► ScenarioOutcome ──► Reports (console/JSON/JUnit/Markdown)
```

For a full integration:

```
crucible-scenarios
   ├── simulator  (adapter over crucible-simulator interfaces)
   ├── prover     (adapter over crucible-prover interfaces)
   └── soroban    (adapter over deployed/local Confidential Token contracts)
              │
              ▼
      Confidential Token
              │
              ▼
         verification
```

## Crate map and dependency direction

```
 scenario-core     domain model (definitions, outcomes, classification)
 scenario-runner   executes scenarios (depends on core)
 scenario-registry discovers/filters scenarios (depends on core)
 fixtures          deterministic test data (depends on core)
 assertions        reusable validation over observations (depends on core)
 adapters/*        connect external systems; implement core's stable contracts
 flows             reusable protocol workflows (compose adapters/assertions)
 negative          expected-rejection utilities
 adversarial       assumption-violation utilities
 conformance       protocol-conformance suites
 invariants        cross-operation properties
 regression        permanent bug regressions
 fuzz              fuzzing orchestration
 reporting         result/report generation
 cli               crucible-scenarios command surface
```

Dependency direction stays approximately:
`scenario-core → runner → scenario implementations`, with adapters consuming
core's stable interfaces and reporting consuming results. No crate depends on
private internals of another Crucible repository.

## Testing model and oracles

Every scenario conceptually follows: definition → initial state → actor setup
→ token setup → operation construction → simulator execution → witness/proof
request → proof generation → proof verification → state transition → event
capture → assertion → invariant validation → result classification → report.

Expected results are derived from declared inputs and **protocol rules**, never
by calling the same internal state-transition function under test. Layered
oracles are used:

1. explicit expected result,
2. protocol invariant,
3. independent state comparison,
4. cross-component comparison.

A single successful return value is never sufficient evidence for an important
scenario.

## See also

- [scenario-model.md](scenario-model.md) — the domain vocabulary.
- [execution-model.md](execution-model.md) — lifecycle and reproducibility.
- [simulator-integration.md](simulator-integration.md) — simulator adapter.
- [prover-integration.md](prover-integration.md) — prover adapter.
- [assertions.md](assertions.md) — assertion library.
- [invariants.md](invariants.md) — cross-operation properties.
