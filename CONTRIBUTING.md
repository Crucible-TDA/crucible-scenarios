# Contributing

Thank you for contributing to `crucible-scenarios`, the STRESS-TEST layer of
the Crucible hybrid system.

## Repository boundaries — read this first

This repository **validates** the Crucible stack; it does not **define** it.

- Do not implement Confidential Token protocol semantics here. If you need
  simulation semantics, consume `crucible-simulator` through
  `crates/adapters/simulator`; if you need proofs, consume `crucible-prover`
  through `crates/adapters/prover`.
- Do not invent fake protocol semantics merely to make a test pass. When an
  expected result derives from a protocol invariant, document the invariant.
- Tests validate **externally observable behavior**. Derive expected results
  from declared inputs and protocol rules — never by calling the same internal
  function under test (anti-circular-testing requirement).
- A single successful return value is never enough for important scenarios:
  layer oracles (explicit expectation → invariant → independent state
  comparison → cross-component comparison).
- Never commit private keys, wallet seeds, private witnesses, or confidential
  user information. Sensitive values stay out of logs, reports, and fixtures.
- Mocks are test doubles, not cryptographic proofs. Label them as such.
- Testnet scenarios are tagged `testnet`, isolated, and never required by
  ordinary CI.

## Adding a scenario

1. Pick a stable scenario ID (e.g. `CT-XFER-001`) — IDs are permanent.
2. Declare initial state, operation sequence, expectations, assertions,
   invariants, timeout, seed, and required capabilities.
3. Prefer the declarative scenario format (see `docs/scenario-format.md`) for
   scenarios that need no custom Rust; use code-defined scenarios for complex
   cases.
4. Register the scenario and give it deterministic fixtures and test vectors.
5. Add unit tests for any new model/utility code and, where possible, a
   regression record for every discovered bug.
6. Verify: `scripts/validate-scenarios.sh`, then the relevant test script
   (e.g. `scripts/test-unit.sh`, `scripts/test-adversarial.sh`).

Contributors can add scenarios without modifying core architecture.

## Development workflow

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Commit messages describe the *what* and the *why*; each commit is one
self-contained improvement. See `CHANGELOG.md` for the format used.

## Issue surface

The repository intentionally provides contributor opportunities across:
scenario implementation, negative/adversarial/conformance/invariant/regression/
privacy tests, fuzz targets, performance scenarios, integration adapters, test
vectors, fixtures, reporting, CLI, and documentation. Every issue identifies a
scenario ID, objective, affected area, expected behavior, implementation
requirements, acceptance criteria, and required tests/docs — see the GitHub
issue templates in `.github/ISSUE_TEMPLATE/`.

## Code of conduct

All contributors are expected to follow the
[Code of Conduct](CODE_OF_CONDUCT.md).
