# CI integration

> **Status: untested.** The steps below follow from the documented exit codes. They have not been run in CI against a real project, and no prebuilt binary exists, so a CI job must build the tool from source. (This repository's own workflow, which checks SorobanShield itself, has run successfully.)

## How the exit codes map to CI

- `0` passes the job.
- `1` fails the job because a finding reached the `--fail-on` severity.
- `2` or `3` also fail the job: the command was invalid or the scan could not be completed. A scan that could not analyze the project never exits `0`.

## Illustrative shell steps

Run these in any CI system that has a Rust toolchain. They are not a verified workflow.

```bash
git clone https://github.com/sorobanshield/soroban-shield.git sorobanshield-src
cargo build --release --manifest-path sorobanshield-src/Cargo.toml
./sorobanshield-src/target/release/sorobanshield scan . --fail-on medium
```

To keep the report as an artifact, use JSON output and keep the exit code:

```bash
./sorobanshield-src/target/release/sorobanshield scan . --format json > sorobanshield-report.json
```

## Cautions

- Pick `--fail-on` deliberately. SS-001 findings are medium severity and low confidence: review prompts, not confirmed bugs.
- Pin the SorobanShield revision you build so results are reproducible.
- A passing job means only that the implemented checks reported nothing.

## Uploading SARIF to GitHub code scanning (untested sketch)

```yaml
- run: sorobanshield scan . --format sarif > sorobanshield.sarif || [ $? -eq 1 ]
- uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: sorobanshield.sarif
```

The `|| [ $? -eq 1 ]` lets findings (exit 1) through to the upload step while still failing the job on exit codes 2 and 3. This snippet has not been run against a real repository.
