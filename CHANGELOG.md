# Changelog

All notable user-visible changes are recorded here. The project follows semantic versioning once a release is published. No release has been published yet.

## Unreleased

### Changed
- SS-001 and SS-005 no longer report `__constructor`, which runs only at deployment. Files declared with `#[cfg(test)] mod name;` are now treated as test-only: per-file rules skip them, and SS-005 reads them as test code. Files under a `tests/` directory are also test-only. Found by scanning stellar/soroban-examples and larger public codebases; see `docs/limitations.md`.
- Documentation now has one canonical copy in `docs/`. The website imports it instead of keeping separate copies that had drifted apart (the repository `limitations` and `cli-reference` had newer content than the site, and vice versa). `docs/quickstart.md` and `docs/reports.md` were added.

### Added
- `CODE_OF_CONDUCT.md`, a pull-request template and issue templates (bug report, rule proposal).
- CI `audit` job (`cargo audit` and `npm audit --omit=dev --audit-level=high`); not yet observed running on GitHub.
- `sorobanshield scan [PATH]` with `--format text|json` and `--fail-on <severity>`.
- `sorobanshield rules list` and `sorobanshield rules explain <ID>`, each with `--format json`.
- Project discovery: requires a `Cargo.toml`, requires a `soroban-sdk` dependency in some manifest, skips `target`, `node_modules`, `.git`, `vendor`, `dist` and `build`.
- Rule `SS-001` (authorization path review, experimental-quality heuristic, medium severity / low confidence).
- Rule `SS-002` (arithmetic boundary review; low severity / low confidence): unchecked `+ - *` and narrowing or sign-changing `as` casts on integer parameters of public contract functions.
- Rule `SS-003` (external contract call review; low severity / low confidence): caller-supplied call targets and discarded results of fallible external calls.
- Rule `SS-004` (error-path review; informational): `unwrap`, `expect`, `panic!`, `todo!` and `unimplemented!` in public contract functions.
- Rule `SS-005` (state-transition test suggestions; informational): state-changing entrypoints that no test code in the project mentions. Rules can now implement `check_project` to see all parsed files together.
- Rule `SS-006` (dependency and toolchain diagnostics; informational): unpinned `soroban-sdk`, missing `cdylib` crate type, release profile without `overflow-checks = true`, missing `Cargo.lock` or `rust-toolchain.toml`. Rules can now inspect parsed `Cargo.toml` files through `Rule::check_manifest`.
- Configuration file `sorobanshield.toml` (or `--config <FILE>`): `[scan] exclude`, `[scan] fail_on`, `[rules] disable`. Strictly validated; invalid configuration exits 2. The JSON report gains `scan.config_file`, `scan.exclude` and `scan.disabled_rules` (additive; schema version stays 1).
- `scan --format sarif`: SARIF 2.1.0 output (severity mapped to SARIF levels; failed scans reported via `invocations`).
- Baselines: `scan --write-baseline FILE` and `scan --baseline FILE`. Entries match on rule, file and message (not line numbers). The JSON report gains `summary.suppressed_by_baseline`, `scan.baseline_file` and `scan.baseline_unmatched` (additive).
- CI job `sarif`: `scripts/check-sarif.sh` validates SARIF for every fixture project against the official OASIS schema (pinned by checksum).
- Inline suppressions: `// sorobanshield-ignore: SS-001 -- reason` above a line or block. Suppressed findings and unused directives are always reported; the JSON report gains `summary.suppressed_by_comment`, `suppressions` and `unused_suppressions` (additive).
- JSON report schema version 1.
- Exit codes: 0 no findings at or above the threshold, 1 findings at or above it, 2 invalid usage, 3 scan could not be completed (including unparseable source files).
- Documentation website under `website/`.
