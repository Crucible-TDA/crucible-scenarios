# Crucible Scenarios

**STRESS-TEST** — the scenario, conformance, adversarial-testing, regression,
invariant, privacy, compatibility, and stress-testing layer of the **Crucible
hybrid system** for Stellar/Soroban **Confidential Tokens**.

```
            CRUCIBLE
               │
   ┌───────────┼───────────┐
   ▼           ▼           ▼
 SIMULATE     PROVE     STRESS-TEST
crucible-   crucible-  crucible-
simulator    prover     scenarios
   │           │           │
   └───────────┼───────────┘
               ▼
    Confidential Token
        validation
```

Crucible consists of exactly three polyrepos:

| Polyrepo | Layer | Responsibility |
| --- | --- | --- |
| `crucible-simulator` | SIMULATE | deterministic simulation and state execution |
| `crucible-prover` | PROVE | circuits, witnesses, proving, verification, proof artifacts |
| `crucible-scenarios` | STRESS-TEST | scenario orchestration, conformance, adversarial, invariant, privacy, regression, fuzzing, stress testing, reporting |

The overall lifecycle is:
**CONSTRUCT → EXECUTE → PROVE → VERIFY → OBSERVE → ASSERT → STRESS-TEST → REPORT**.

## What this repository does

`crucible-scenarios` answers one question:

> Given a known initial state, a defined Confidential Token workflow, a proof
> provider, and expected protocol behavior, does the complete system behave
> correctly under normal, invalid, adversarial, privacy-sensitive,
> state-sensitive, concurrent, compatibility, and high-load conditions?

It owns:

- scenario definitions, execution, and orchestration
- deterministic fixtures and test vectors
- assertions, expected outcomes, and invariants
- failure classification and regression cases
- negative, adversarial, privacy, conformance, and compatibility suites
- stress, concurrency, performance, and fuzzing orchestration
- structured reporting (console, JSON, JUnit, Markdown) and reproducible replay

## What this repository does NOT do

It **must not** become another simulator, another prover, another wallet,
another token implementation, or a generic testing framework. Specifically it
does **not** own: token contract implementation, Confidential Token
cryptography, proof/circuit implementation, wallet or production account
management, compliance/sanctions policy, audit dashboards, or production
transaction orchestration.

It consumes stable interfaces from `crucible-simulator` and `crucible-prover`
through the adapter crates in [`crates/adapters/`](crates/adapters/). It never
re-implements what those repositories own, never reaches into their private
internals, and never treats mocked proofs as cryptographically valid proofs.

## Repository layout

```
crates/           scenario-core, scenario-runner, scenario-registry, fixtures,
                  assertions, adapters/{simulator,prover,soroban,testnet},
                  flows, negative, adversarial, conformance, invariants,
                  regression, fuzz, reporting
scenarios/        declarative + code scenario definitions by family
test-vectors/     versioned test vectors by operation
fixtures/         deterministic accounts, tokens, states, proofs, events
schemas/          versioned JSON schemas (scenario, result, vector, ...)
examples/         runnable worked examples
cli/              crucible-scenarios command-line interface
benches/          criterion benches
fuzz/             cargo-fuzz targets and corpora
docs/             architecture and testing-model documentation
scripts/          test/validate/benchmark helpers
.github/          layered CI workflows and issue templates
```

See [`docs/architecture.md`](docs/architecture.md) for the detailed design.

## Scenario lifecycle

Every scenario conceptually follows:

```
Scenario Definition → Initial State → Actor Setup → Token Setup
→ Operation Construction → Simulator Execution → Witness/Proof Request
→ Proof Generation → Proof Verification → State Transition → Event Capture
→ Assertion → Invariant Validation → Result Classification → Report
```

## Quick start

Requires a stable Rust toolchain (see `rust-toolchain.toml`).

```bash
cargo build --workspace
cargo test  --workspace
```

The repository is under construction; the scenario model, runner, registry,
fixtures, assertions, adapters, scenario suites, reporting, CLI, and CI layers
are introduced incrementally. See `CHANGELOG.md` for the current state.

## Relationship to the other Crucible repositories

- `crates/adapters/simulator` connects scenarios to `crucible-simulator`.
- `crates/adapters/prover` connects scenarios to `crucible-prover`.
- `crates/adapters/soroban` invokes deployed/local Confidential Token contracts
  through stable interfaces.
- `crates/adapters/testnet` is optional, explicitly configured, isolated, and
  never required by ordinary CI.

## Contributing

Scenarios must validate externally observable behavior, derive expected values
from declared inputs and protocol rules (never from the same internal function
under test), and must not leak private witnesses. See
[`CONTRIBUTING.md`](CONTRIBUTING.md) and [`docs/contributing.md`](docs/contributing.md).

## License

Apache-2.0 — see [`LICENSE`](LICENSE).
