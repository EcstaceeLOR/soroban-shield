# CI integration

This repository's own workflow is `.github/workflows/ci.yml` (rustfmt, clippy, tests, website lint/typecheck/test/build). Its first run on `main` passed.

To gate a project on SorobanShield you need the binary in CI. Since no release exists, build it from source in the job. The following is an untested sketch:

```yaml
- uses: dtolnay/rust-toolchain@stable
- run: cargo install --git https://github.com/sorobanshield/soroban-shield sorobanshield --locked
- run: sorobanshield scan . --fail-on medium
```

Exit code 1 fails the job when findings meet the threshold; exit code 3 also fails it, so a scan that could not run is never mistaken for a clean one. Use `--format json` to store the report as an artifact.

## Uploading SARIF to GitHub code scanning (untested sketch)

```yaml
- run: sorobanshield scan . --format sarif > sorobanshield.sarif || [ $? -eq 1 ]
- uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: sorobanshield.sarif
```

The `|| [ $? -eq 1 ]` lets findings (exit 1) through to the upload step while still failing the job on exit codes 2 and 3. This snippet has not been run against a real repository.
