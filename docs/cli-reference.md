# CLI reference

```text
sorobanshield --help
sorobanshield --version
sorobanshield scan [PATH] [--format text|json|sarif] [--fail-on info|low|medium|high|critical] [--config FILE] [--baseline FILE | --write-baseline FILE]
sorobanshield rules list [--format text|json]
sorobanshield rules explain <RULE_ID> [--format text|json]
```

`PATH` defaults to `.`. `--format` defaults to `text`; `sarif` is available for `scan` only. `--fail-on` defaults to `low` unless the config file sets `fail_on`; the flag wins over the config file. `--config` defaults to `<PATH>/sorobanshield.toml` when present. See [configuration.md](configuration.md). Rule IDs are case-insensitive.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Scan completed; no finding at or above `--fail-on` |
| 1 | Scan completed; at least one finding at or above `--fail-on` |
| 2 | Invalid usage or configuration (unknown command or option, bad `--fail-on`, unknown rule ID, missing or invalid config file) |
| 3 | Scan could not be completed: path missing or not a directory, missing/malformed `Cargo.toml`, no `soroban-sdk` dependency, no `.rs` files, or a source file that could not be read or parsed |

Errors go to stderr. In JSON mode stdout contains only the JSON document when a report was produced; discovery failures produce no JSON, only stderr and exit 3.

## JSON report (schema_version 1)

```json
{
  "schema_version": 1,
  "tool": { "name": "sorobanshield", "version": "0.1.0" },
  "scan": { "root": ".", "files_scanned": 1, "rules_run": ["SS-001", "SS-002", "SS-006"], "config_file": null, "exclude": [], "disabled_rules": [], "baseline_file": null, "baseline_unmatched": 0, "errors": [] },
  "summary": { "total": 1, "by_severity": { "info": 0, "low": 0, "medium": 1, "high": 0, "critical": 0 } },
  "findings": [
    {
      "rule_id": "SS-001", "title": "...", "severity": "medium", "confidence": "low",
      "message": "...", "file": "src/lib.rs", "start_line": 10, "end_line": 10,
      "remediation": "...", "references": ["..."], "limitations": ["..."]
    }
  ],
  "disclaimer": "..."
}
```

`file` is relative to the scan root with `/` separators. `scan.errors` lists files that could not be analyzed (`{file, message}`); a non-empty list means the scan is incomplete and the exit code is 3. Output is deterministic for identical input.

## Baselines

A baseline lets a team adopt scanning gradually: record today's findings once, then fail only on new ones.

```bash
sorobanshield scan . --write-baseline baseline.json   # records findings; exits 0 if the scan completed
sorobanshield scan . --baseline baseline.json         # hides recorded findings; exits 1 only for new ones
```

Each entry identifies a finding by rule ID, file and message, never by line number, so entries survive unrelated edits that move code. Two findings with the same rule, file and message share one entry. The two flags cannot be combined. A missing or invalid baseline file is a usage error (exit 2), and no baseline is written if any file could not be analyzed (exit 3).

Suppression is visible: the text report prints a `baseline:` line, and the JSON report sets `summary.suppressed_by_baseline`, `scan.baseline_file` and `scan.baseline_unmatched`. A non-zero `baseline_unmatched` means some entries no longer match any finding (the issue was fixed or its message changed) and can be regenerated. Baselines hide findings; they do not mean the findings were reviewed or are safe. Commit the baseline and review changes to it like code.

## Inline suppressions

A comment of the form `// sorobanshield-ignore: SS-001, SS-003 -- reason` hides findings of the named rules on the comment's own line or on the next line of code (blank lines, other comments and `#[...]` attributes are skipped). When that line opens a `{` block, such as a function, the whole block is covered; the end of the block is found by counting braces outside string literals, so it is approximate. The reason after `--` is optional but recommended.

Suppression is never silent. The text report prints an `inline suppressions:` list with each hidden finding and its reason. The JSON report sets `summary.suppressed_by_comment` and lists every hidden finding in `suppressions` (`rule_id`, `file`, `line`, `reason`). A rule ID that matched no finding is reported as a warning and in `unused_suppressions`, so stale comments can be cleaned up.

Limits: this works for Rust source findings only. Manifest and project-wide findings (SS-005, SS-006) have no source line to attach to; use `[rules] disable` or a baseline for those. The comment is found by a line scan, so a `//` inside a string literal that contains the marker is also read as a directive. A suppression records that someone chose to hide a finding. It does not mean the finding was reviewed or is safe.

## SARIF output

`sorobanshield scan --format sarif` writes a SARIF 2.1.0 log for code-scanning tools. Severity maps to the SARIF `level`: `critical` and `high` become `error`, `medium` becomes `warning`, and `low` and `info` become `note`. The original severity and confidence are kept in each result's `properties`. Findings carry a file URI relative to the scan root (`uriBaseId` `%SRCROOT%`) and, when known, a line region; manifest findings have no region. `invocations[0].executionSuccessful` is `false`, with a notification per file, when files could not be analyzed.

SARIF output for every fixture project in this repository is validated against the official OASIS SARIF 2.1.0 JSON schema by `scripts/check-sarif.sh`, which CI runs. SARIF is only available for `scan`, not for `rules list` or `rules explain`.
