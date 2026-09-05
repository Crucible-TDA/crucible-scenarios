# Changelog

All notable changes to `crucible-scenarios` are documented here. The format is
informed by [Keep a Changelog](https://keepachangelog.com/), and this project
adheres to [Semantic Versioning](https://semver.org/).

Every entry records the *what* and the *why* of a change. Scenario and test
vector IDs, once shipped, are stable.

## [Unreleased]

### Added (phase 1 — scenario-core)

- Repository baseline: workspace manifest, toolchain pins, formatting/lint
  configuration, license, and hygiene documentation (README, SECURITY,
  CONTRIBUTING, CODE_OF_CONDUCT).
- `scenario-core` is introduced as the domain model crate: scenario identity,
  metadata, actors, environments, capabilities, operations, expectations,
  observations, assertions, outcomes, failures, severity, tags, seeds, and the
  scenario context that carries stable integration interfaces. Details land in
  their own commits.

No release has been cut yet; the crate inventory, scenario suites, reporting,
CLI, and CI layers are built out incrementally in later phases.
