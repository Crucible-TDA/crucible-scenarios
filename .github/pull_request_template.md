## What and why

Describe the improvement and the reason for it (one or two sentences). Each PR
should be one self-contained improvement with a matching `CHANGELOG.md` entry
when user-visible.

## Boundary checklist

- [ ] No simulator, prover, wallet, token, or protocol semantics were
      implemented or duplicated in this repository.
- [ ] Expected results come from declared inputs and protocol rules — not from
      calling the same internal function under test.
- [ ] No private witnesses, keys, seeds, or confidential amounts are committed,
      logged, or emitted in reports.
- [ ] Mocked proofs are labeled as test doubles and never treated as
      cryptographically valid proofs.
- [ ] Testnet scenarios are tagged `testnet`, isolated, and not required by
      this CI path.

## Scenarios / IDs affected

List scenario IDs, test vector IDs, fixture keys, or regression IDs touched.

## Tests

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`

## Documentation

- [ ] Docs updated (`docs/`), including the scenario/vector schema when it
      changed.

## Acceptance criteria

State how a reviewer can verify this change does what it claims.
