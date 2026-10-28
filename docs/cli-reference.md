# CLI reference

```text
Security review toolkit for Soroban smart contracts

Usage: sorobanshield <COMMAND>

Commands:
  scan   Scan a Soroban project directory
  rules  Inspect the rule catalog
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

Exit codes: 0 no findings at or above --fail-on; 1 findings at or above --fail-on; 2 invalid usage or configuration; 3 scan could not be completed.
Static analysis cannot prove a contract is secure.
```

## `sorobanshield --version`

Prints `sorobanshield <version>` and exits `0`.

## `sorobanshield scan [PATH]`

Scans a Soroban project. `PATH` defaults to `.` and must be a project root containing `Cargo.toml` (see [supported layout](installation.md)).

| Option | Values | Default | Meaning |
| --- | --- | --- | --- |
| `--format` | `text`, `json`, `sarif` | `text` | Output format on stdout. SARIF is available for `scan` only. |
| `--fail-on` | `info`, `low`, `medium`, `high`, `critical` | `low` (or `fail_on` from the config file) | Exit `1` when any finding has at least this severity. The flag overrides the config file. |
| `--baseline` | file path | none | Hide findings recorded in a baseline file; see [Reports](reports.md#baselines). |
| `--write-baseline` | file path | none | Record current findings to a baseline file; exits `0` if the scan completed. Cannot be combined with `--baseline`. |
| `--recursive` | flag | off | Scan every Soroban project below `PATH` instead of requiring `PATH` itself to be one. Each project is analysed on its own and reported paths stay relative to `PATH` (for example `alpha/src/lib.rs`). A directory that is already a project, such as a workspace root, is one project and is not searched further. `target`, `node_modules`, `.git`, `vendor`, `dist`, `build` and configured excludes are skipped. A project whose manifest cannot be read or parsed fails the scan (exit `3`) rather than being skipped, and finding no project is also exit `3`. |
| `--config` | file path | `<PATH>/sorobanshield.toml` if it exists | Configuration file; see [Configuration](configuration.md). |

Errors are written to stderr. With `--format json`, stdout contains only the JSON document.

```bash
sorobanshield scan .
sorobanshield scan . --format json
sorobanshield scan . --fail-on high
sorobanshield scan path/to/examples --recursive
```

## `sorobanshield rules list [--format text|json]`

Lists every rule in the catalog with its status. Only rules marked `implemented` run during a scan; all current rules are implemented.

```text
SS-001  [implemented] Authorization path review
SS-002  [implemented] Arithmetic boundary review
SS-003  [implemented] External contract call review
SS-004  [implemented] Error-path review
SS-005  [implemented] State-transition test suggestions
SS-006  [implemented] Dependency and toolchain diagnostics
```

## `sorobanshield rules explain <RULE_ID> [--format text|json]`

Prints a rule's concern, detection method, severity rationale, remediation, known false positives and false negatives, references and fixtures. Rule IDs are matched case-insensitively. An unknown ID exits `2`.

## Exit codes

| Code | Meaning |
| --- | --- |
| `0` | The scan completed and no finding reached the `--fail-on` severity (or a `rules` command succeeded). |
| `1` | The scan completed and at least one finding reached the `--fail-on` severity. |
| `2` | Invalid usage or configuration: unknown command, bad option value, unknown rule ID, or a missing or invalid config file. |
| `3` | The scan could not be completed: path missing or not a directory, no `Cargo.toml`, malformed manifest, not a Soroban project, no Rust files, or a source file that could not be read or parsed. |

A scan that could not analyze every file exits `3` even if no findings were found. It is never reported as clean.
