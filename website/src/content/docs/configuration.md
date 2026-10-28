# Configuration

SorobanShield reads an optional TOML file. By default it uses `<PATH>/sorobanshield.toml` when that file exists; pass `--config <FILE>` to use another file. An explicit `--config` that does not exist is an error. A config file that cannot be read or is invalid is always an error (exit code 2): it is never ignored, and a scan never silently falls back to defaults.

```toml
[scan]
# Project-relative paths (files or directories) to skip. Plain path prefixes, not globs.
exclude = ["vendor", "tests/generated"]
# Exit with code 1 when a finding has at least this severity.
# One of: info, low, medium, high, critical. The --fail-on flag overrides it. Default: low.
fail_on = "medium"

[rules]
# Rule IDs that should not run. Only implemented rules may be named.
disable = ["SS-002"]
```

## Validation

- Unknown keys and sections are rejected (a misspelled `exlude` is an error, not ignored).
- `fail_on` must be a valid severity.
- `exclude` entries must be relative, use `/` separators, contain no `..`, and contain no glob characters (`* ? [ ] { }`). Globs are rejected rather than silently ignored.
- `disable` entries must name a rule that exists and can run (matching is case-insensitive). Disabling every rule is an error.

## Visibility

When a config file is applied, the report says so: the text report prints a `config:` line, and the JSON report sets `scan.config_file`, `scan.exclude` and `scan.disabled_rules`. `scan.rules_run` lists only rules that actually ran.

## What is not configurable

Output format (use `--format`), the directories skipped by default (`target`, `node_modules`, `.git`, `vendor`, `dist`, `build`), and per-rule severity. Individual findings are suppressed with baselines or inline `sorobanshield-ignore` comments (see the CLI reference), not with configuration.

There are no environment variables, and SorobanShield does not read `.env` files.
