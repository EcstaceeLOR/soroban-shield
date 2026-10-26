# Reports

SorobanShield prints a human-readable text report, a JSON document or a SARIF 2.1.0 log. All carry the same findings.

## Text report

```text
SorobanShield 0.1.0 - scanned 2 file(s) with 6 rule(s)

[1] SS-001 Authorization path review (severity: medium, confidence: low)
    at src/lib.rs:10
    ...
    next step: ...
    limitation: ...

1 finding(s). Static analysis cannot prove a contract is secure. ...
```

Each finding shows the rule ID, severity, confidence, location, message, a suggested next step and the rule's limitations. If files could not be analyzed, the report lists them and says the result is **not** a clean result.

## JSON report

Run `sorobanshield scan . --format json`. Key order is fixed and the output is deterministic for the same input.

| Field | Type | Notes |
| --- | --- | --- |
| `schema_version` | number | Currently `1`. |
| `tool` | object | `name` and `version`. |
| `scan.root` | string | The path you passed. |
| `scan.files_scanned` | number | Files parsed successfully. |
| `scan.rules_run` | string[] | IDs of rules that ran. |
| `scan.config_file` | string or null | Config file that was applied, if any. |
| `scan.exclude` | string[] | Path prefixes excluded by configuration. |
| `scan.baseline_file` | string or null | Baseline file that was applied, if any. |
| `scan.baseline_unmatched` | integer | Baseline entries that matched no current finding. |
| `scan.disabled_rules` | string[] | Rules disabled by configuration. |
| `scan.errors` | object[] | `{ file, message }` for files that could not be read or parsed. |
| `summary.total` | number | Number of findings. |
| `summary.by_severity` | object | Counts for `critical`, `high`, `info`, `low`, `medium`. |
| `findings` | object[] | Sorted by file, line, rule ID. |
| `disclaimer` | string | Reminder that static analysis cannot prove security. |

Each finding has `rule_id`, `title`, `severity`, `confidence`, `message`, `file` (relative, `/`-separated), `start_line`, `end_line` (both may be `null`), `remediation`, `references` and `limitations`.

```json
{
  "schema_version": 1,
  "tool": { "name": "sorobanshield", "version": "0.1.0" },
  "scan": { "root": ".", "files_scanned": 1, "rules_run": ["SS-001", "SS-002", "SS-003", "SS-004", "SS-005", "SS-006"], "config_file": null, "exclude": [], "disabled_rules": [], "baseline_file": null, "baseline_unmatched": 0, "errors": [] },
  "summary": { "total": 0, "by_severity": { "critical": 0, "high": 0, "info": 0, "low": 0, "medium": 0 } },
  "findings": [],
  "disclaimer": "Static analysis cannot prove a contract is secure. ..."
}
```

Severity (potential impact) and confidence (how sure the rule is that the pattern matters) are separate fields. A fatal error such as a missing path prints a message to stderr and no JSON; check the exit code.

## SARIF output

`sorobanshield scan --format sarif` writes a SARIF 2.1.0 log for code-scanning tools. Severity maps to the SARIF `level`: `critical` and `high` become `error`, `medium` becomes `warning`, and `low` and `info` become `note`. The original severity and confidence are kept in each result's `properties`. Findings carry a file URI relative to the scan root (`uriBaseId` `%SRCROOT%`) and, when known, a line region; manifest findings have no region. `invocations[0].executionSuccessful` is `false`, with a notification per file, when files could not be analyzed.

The output of this repository's fixtures was validated against the official OASIS SARIF 2.1.0 JSON schema during development. That validation is a manual step and is not part of CI. SARIF is only available for `scan`, not for `rules list` or `rules explain`.

## Baselines

A baseline lets a team adopt scanning gradually: record today's findings once, then fail only on new ones.

```bash
sorobanshield scan . --write-baseline baseline.json   # records findings; exits 0 if the scan completed
sorobanshield scan . --baseline baseline.json         # hides recorded findings; exits 1 only for new ones
```

Each entry identifies a finding by rule ID, file and message, never by line number, so entries survive unrelated edits that move code. Two findings with the same rule, file and message share one entry. The two flags cannot be combined. A missing or invalid baseline file is a usage error (exit 2), and no baseline is written if any file could not be analyzed (exit 3).

Suppression is visible: the text report prints a `baseline:` line, and the JSON report sets `summary.suppressed_by_baseline`, `scan.baseline_file` and `scan.baseline_unmatched`. A non-zero `baseline_unmatched` means some entries no longer match any finding (the issue was fixed or its message changed) and can be regenerated. Baselines hide findings; they do not mean the findings were reviewed or are safe. Commit the baseline and review changes to it like code.
