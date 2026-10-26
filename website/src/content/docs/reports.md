# Reports

SorobanShield prints either a human-readable text report or a JSON document. Both carry the same findings. SARIF is **planned** and not implemented.

## Text report

```text
SorobanShield 0.1.0 - scanned 1 file(s) with 2 rule(s)

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
  "scan": { "root": ".", "files_scanned": 1, "rules_run": ["SS-001", "SS-002"], "errors": [] },
  "summary": { "total": 0, "by_severity": { "critical": 0, "high": 0, "info": 0, "low": 0, "medium": 0 } },
  "findings": [],
  "disclaimer": "Static analysis cannot prove a contract is secure. ..."
}
```

Severity (potential impact) and confidence (how sure the rule is that the pattern matters) are separate fields. A fatal error such as a missing path prints a message to stderr and no JSON; check the exit code.
