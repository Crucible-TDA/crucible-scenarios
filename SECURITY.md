# Security Policy

## Reporting a vulnerability

This repository is a testing and validation layer. It never contains production
credentials, wallet seeds, private keys, or confidential user data by design —
but a defect in scenario, fixture, or reporting code could still leak sensitive
material or mis-validate the Crucible stack.

If you believe you have found a security vulnerability in this repository or in
anything it validates, please report it privately. Do **not** open a public
issue.

- Open a **private security advisory** on GitHub for this repository, or
- Contact the maintainers through the contact channels listed in
  `CONTRIBUTING.md`.

Include, when possible:

- the affected crate or scenario family and version,
- a minimal reproduction (seed, scenario ID, fixture, or sequence),
- the observed versus expected behavior,
- whether any sensitive material was involved.

## Scope

In scope:

- accidental disclosure of witness/secret material through logs, errors,
  reports, fixtures, or serialized results,
- scenario or assertion logic that accepts invalid proofs, stale state, or
  unauthorized operations,
- repository-boundary violations (e.g. re-implementing simulator/prover
  semantics), and
- redaction or classification failures in observation/reporting code.

Out of scope:

- Confidential Token protocol design itself (owned by the other Crucible
  polyrepos), and
- proofs/circuits/witnesses produced by `crucible-prover`.

## Security expectations

- No production secrets are ever committed.
- Private witnesses and confidential amounts never appear in logs or reports.
- Mocks are never treated as cryptographically valid proofs.
- Testnet execution is optional, isolated, and never required by ordinary CI.

See [`docs/security.md`](docs/security.md) for the testing-security model.
