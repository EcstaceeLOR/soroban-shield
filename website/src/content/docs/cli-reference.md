# CLI reference

```text
Usage: sorobanshield <COMMAND>

Commands:
  scan   Scan a Soroban project directory
  rules  Inspect the rule catalog
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## `sorobanshield --version`

Prints `sorobanshield <version>` and exits `0`.

## `sorobanshield scan [PATH]`

Scans a Soroban project. `PATH` defaults to `.` and must be a project root containing `Cargo.toml` (see [supported layout](/docs/installation)).

| Option | Values | Default | Meaning |
| --- | --- | --- | --- |
| `--format` | `text`, `json` | `text` | Output format on stdout. |
| `--fail-on` | `info`, `low`, `medium`, `high`, `critical` | `low` | Exit `1` when any finding has at least this severity. |

Errors are written to stderr. With `--format json`, stdout contains only the JSON document.

```bash
sorobanshield scan .
sorobanshield scan . --format json
sorobanshield scan . --fail-on high
```

## `sorobanshield rules list [--format text|json]`

Lists every rule in the catalog, including planned rules. Planned rules are metadata only and are never run by `scan`.

```text
SS-001  [implemented] Authorization path review
SS-002  [implemented] Arithmetic boundary review
SS-003  [planned]     External contract call review
SS-004  [planned]     Error-path review
SS-005  [planned]     State-transition test suggestions
SS-006  [implemented] Dependency and toolchain diagnostics
```

## `sorobanshield rules explain <RULE_ID> [--format text|json]`

Prints a rule's concern, detection method, severity rationale, remediation, known false positives and false negatives, references and fixtures. Rule IDs are matched case-insensitively. An unknown ID exits `2`.

## Exit codes

| Code | Meaning |
| --- | --- |
| `0` | The scan completed and no finding reached the `--fail-on` severity (or a `rules` command succeeded). |
| `1` | The scan completed and at least one finding reached the `--fail-on` severity. |
| `2` | Invalid usage: unknown command, bad option value, or unknown rule ID. |
| `3` | The scan could not be completed: path missing or not a directory, no `Cargo.toml`, malformed manifest, not a Soroban project, no Rust files, or a source file that could not be read or parsed. |

A scan that could not analyze every file exits `3` even if no findings were found. It is never reported as clean.
