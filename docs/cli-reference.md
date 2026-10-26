# CLI reference

```text
sorobanshield --help
sorobanshield --version
sorobanshield scan [PATH] [--format text|json] [--fail-on info|low|medium|high|critical]
sorobanshield rules list [--format text|json]
sorobanshield rules explain <RULE_ID> [--format text|json]
```

`PATH` defaults to `.`. `--format` defaults to `text`; `--fail-on` defaults to `low`. Rule IDs are case-insensitive.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Scan completed; no finding at or above `--fail-on` |
| 1 | Scan completed; at least one finding at or above `--fail-on` |
| 2 | Invalid usage (unknown command or option, bad `--fail-on`, unknown rule ID) |
| 3 | Scan could not be completed: path missing or not a directory, missing/malformed `Cargo.toml`, no `soroban-sdk` dependency, no `.rs` files, or a source file that could not be read or parsed |

Errors go to stderr. In JSON mode stdout contains only the JSON document when a report was produced; discovery failures produce no JSON, only stderr and exit 3.

## JSON report (schema_version 1)

```json
{
  "schema_version": 1,
  "tool": { "name": "sorobanshield", "version": "0.1.0" },
  "scan": { "root": ".", "files_scanned": 1, "rules_run": ["SS-001"], "errors": [] },
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
