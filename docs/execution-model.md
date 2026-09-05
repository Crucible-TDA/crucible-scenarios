# Execution model

Execution is owned by `crates/scenario-runner` (introduced after
`scenario-core`). This document records the model the runner implements.

## Lifecycle

Every scenario follows the same stages; a failure at any stage is classified
with the stage it happened in:

1. **Discover** — find the scenario through the registry.
2. **Validate** — the scenario was already validated at build time; re-check
   against the current context (capabilities, environment, isolation).
3. **Prepare** — prepare environment/fixtures; nothing here may run the
   scenario early.
4. **Initialize** — build the `ScenarioContext`, load initial state, seed the
   deterministic stream.
5. **Execute** — run the ordered operations through the context's services.
6. **Observe** — capture results as classified observations.
7. **Assert** — evaluate the scenario's assertions.
8. **Invariants** — evaluate declared cross-operation invariants.
9. **Classify** — produce the `ScenarioOutcome` with a `Status` and, on
   failure, a classified `Failure`.
10. **Report** — hand the outcome to reporting.
11. **Cleanup** — tear down in all paths.

## Determinism and replay

- Every randomized scenario carries a `Seed`; the seed is reported with the
  outcome.
- Consumers derive isolated child seeds (`ScenarioContext::seed_for`) so
  fixtures, generators, and sequences never interfere.
- Replay (`crucible-scenarios replay --scenario <ID> --seed <SEED>`)
  reconstructs the same fixtures, sequence, and randomized values.
- Replay output never contains secrets.

## Retries

Retries are permitted only where a scenario explicitly allows them, and must
never hide nondeterministic failures: for deterministic scenarios a retry is
normally suspicious. Security-sensitive failures (authorization, privacy,
unexpected acceptance, verification) must never be retried away.

## Parallel execution vs. protocol concurrency

The runner distinguishes *parallel execution* (multiple scenarios run in
parallel threads — an orchestration concern) from *valid protocol
concurrency* (operations racing against the same state — a scenario concern
requiring the `concurrency` capability). Parallel execution is never assumed
to be semantically safe on its own; expected behavior is defined per
scenario.

## Timeouts and cancellation

Each scenario may declare a positive timeout (milliseconds). A run exceeding
it is classified `TIMEOUT` at the executing stage. Runs may be cancelled,
classified `CANCELLED`. Both are distinct from `FAIL` and from `ERROR`
(harness problems) so reports can tell them apart.

## Failure classification

`FailureCategory` distinguishes harness defects (scenario-definition, fixture,
environment, infrastructure) from findings about the system under test
(assertion, invariant, proof, verification, state, authorization, privacy,
compatibility, unexpected acceptance). Security-sensitive categories default
to elevated severities and are flagged, so they can never be buried.

## Statuses

`PASS`, `FAIL`, `SKIPPED`, `EXPECTED_FAILURE`, `ERROR`, `TIMEOUT`,
`CANCELLED`. `EXPECTED_FAILURE` (the declared failure occurred exactly as
declared) counts as a pass; `SKIPPED` is the honest result of a capability or
environment mismatch, never a silent mis-execution.
