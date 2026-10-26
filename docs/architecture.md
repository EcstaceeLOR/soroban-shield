# Architecture

```text
CLI (clap) -> Project::discover -> parse each .rs with syn -> run active rules -> Report -> text | json
```

| Crate | Responsibility |
|---|---|
| `sorobanshield-core` | `Finding`/`Severity`/`Confidence`, `Rule` trait and `RuleMeta`, `Project` discovery, `scan_project` engine, `Report` (JSON and terminal) |
| `sorobanshield-rules` | Rule implementations (`ss001` to `ss006`), shared helpers (`common`), `active_rules()` and `catalog()` |
| `sorobanshield-cli` | Argument parsing, exit codes, output; integration tests in `crates/sorobanshield-cli/tests/cli.rs` |

Design points:

- **Local and read-only.** Discovery and analysis read files only. No project code, build script or `cargo` is run, and there are no network calls.
- **Failure is never clean.** Discovery errors and unparseable files are reported and produce exit code 3.
- **Deterministic output.** Files are visited in sorted order, findings are sorted by file/line/rule, JSON uses ordered maps, and paths are project-relative with `/` separators.
- **Single source of rule metadata.** `RuleMeta` feeds `rules list/explain` and the website catalog.
- **Three kinds of input.** Rules may implement `check` (one parsed Rust file), `check_project` (all parsed files together, used by SS-005 to find tests in other files) and/or `check_manifest` (parsed `Cargo.toml`, plus whether sibling files such as `Cargo.lock` exist). All default to no findings. Manifest findings have no line numbers.
- **Catalog equals registry.** `catalog()` is derived from `active_rules()`, so the catalog, `rules list` and the website can only show rules that actually run.
- **Spans.** Line numbers come from `proc-macro2` with the `span-locations` feature.
- **Sync, single-threaded.** Scans are small and CPU-light, so there is no async runtime.
