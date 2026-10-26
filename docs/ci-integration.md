# CI integration

This repository's own workflow is `.github/workflows/ci.yml` (rustfmt, clippy, tests, website lint/typecheck/test/build). **It has not yet been run on GitHub**, so treat it as untested until a run is green.

To gate a project on SorobanShield you need the binary in CI. Since no release exists, build it from source in the job. The following is an untested sketch:

```yaml
- uses: dtolnay/rust-toolchain@stable
- run: cargo install --git https://github.com/sorobanshield/soroban-shield sorobanshield --locked
- run: sorobanshield scan . --fail-on medium
```

Exit code 1 fails the job when findings meet the threshold; exit code 3 also fails it, so a scan that could not run is never mistaken for a clean one. Use `--format json` to store the report as an artifact.
