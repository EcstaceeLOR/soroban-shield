# Changelog

All notable user-visible changes are recorded here. The project follows semantic versioning once a release is published. No release has been published yet.

## Unreleased

### Added
- `sorobanshield scan [PATH]` with `--format text|json` and `--fail-on <severity>`.
- `sorobanshield rules list` and `sorobanshield rules explain <ID>`, each with `--format json`.
- Project discovery: requires a `Cargo.toml`, requires a `soroban-sdk` dependency in some manifest, skips `target`, `node_modules`, `.git`, `vendor`, `dist` and `build`.
- Rule `SS-001` (authorization path review, experimental-quality heuristic, medium severity / low confidence).
- Rule `SS-002` (arithmetic boundary review; low severity / low confidence): unchecked `+ - *` and narrowing or sign-changing `as` casts on integer parameters of public contract functions.
- Rule `SS-006` (dependency and toolchain diagnostics; informational): unpinned `soroban-sdk`, missing `cdylib` crate type, release profile without `overflow-checks = true`, missing `Cargo.lock` or `rust-toolchain.toml`. Rules can now inspect parsed `Cargo.toml` files through `Rule::check_manifest`.
- Configuration file `sorobanshield.toml` (or `--config <FILE>`): `[scan] exclude`, `[scan] fail_on`, `[rules] disable`. Strictly validated; invalid configuration exits 2. The JSON report gains `scan.config_file`, `scan.exclude` and `scan.disabled_rules` (additive; schema version stays 1).
- `scan --format sarif`: SARIF 2.1.0 output (severity mapped to SARIF levels; failed scans reported via `invocations`).
- JSON report schema version 1.
- Exit codes: 0 no findings at or above the threshold, 1 findings at or above it, 2 invalid usage, 3 scan could not be completed (including unparseable source files).
- Documentation website under `website/`.
