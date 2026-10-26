# Configuration

**SorobanShield does not support a configuration file yet.** There is no `.sorobanshield.toml`, no rule enable/disable switch and no exclusion list in the current version. A config file is on the [roadmap](/roadmap) and is not implemented.

The only options today are command-line flags:

- `--format text|json` on `scan`, `rules list` and `rules explain`
- `--fail-on <severity>` on `scan` (default `low`)

There are no environment variables. SorobanShield does not read `.env` files and does not make network requests.

## Fixed behaviour

- Rules: every rule with status `implemented` runs on every scan.
- Skipped directories: `target`, `node_modules`, `.git`, `vendor`, `dist` and `build`.
- Files scanned: `.rs` files only.
