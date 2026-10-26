# SorobanShield

A security review toolkit for [Soroban](https://developers.stellar.org/docs/build/smart-contracts) smart contracts. It is a Rust command-line tool that parses contract source and points out code patterns that deserve closer human review.

> **Static analysis cannot prove a contract is secure.** A scan with no findings only means the implemented checks reported nothing. SorobanShield complements tests, code review, threat modelling and independent audits; it does not replace them. No audit of SorobanShield itself has been performed, and no release has been published.

## Status

| Area | Status |
|---|---|
| CLI (`scan`, `rules list`, `rules explain`) | Implemented, tested |
| Project discovery and validation | Implemented, tested |
| Terminal and JSON reports (schema v1) | Implemented, tested |
| Rule `SS-001` authorization path review | Implemented (heuristic, medium severity / low confidence) |
| Rule `SS-002` arithmetic boundary review | Implemented (heuristic, low severity / low confidence) |
| Rule `SS-006` dependency and toolchain diagnostics | Implemented (informational, reads `Cargo.toml` and project files only) |
| Rules `SS-003` to `SS-005` | Planned, not run by the scanner |
| Configuration file (`sorobanshield.toml`: exclude paths, disable rules, `fail_on`) | Implemented, strictly validated |
| SARIF output | Not implemented |
| Documentation website (`website/`) | In progress, see `website/` |
| CI workflow | `.github/workflows/ci.yml`; first run on `main` passed |
| Published release / binaries / crates.io | None |
| Independent security review | Not performed |

## Build from source

Prerequisite: a stable Rust toolchain (developed on 1.99; older versions are untested).

```bash
git clone https://github.com/sorobanshield/soroban-shield.git
cd soroban-shield
cargo build --release
./target/release/sorobanshield --help
```

## Usage

```bash
sorobanshield scan .                       # human-readable report
sorobanshield scan . --format json         # machine-readable report (schema v1)
sorobanshield scan . --fail-on medium      # only exit 1 for medium or higher findings
sorobanshield scan . --config ci.toml      # use an explicit config file (see docs/configuration.md)
sorobanshield rules list                   # all rules; planned ones are labelled
sorobanshield rules explain SS-001         # concern, detection, limitations
```

Exit codes: `0` no findings at or above `--fail-on` (default `low`); `1` findings at or above it; `2` invalid usage, invalid configuration or unknown rule; `3` the scan could not be completed (bad path, missing or malformed `Cargo.toml`, not a Soroban project, or a source file that could not be read or parsed). A failed scan never exits `0`.

Try it on a bundled fixture:

```bash
cargo run -q -- scan tests/fixtures/positive/unprotected_write   # exits 1 with one SS-001 finding
cargo run -q -- scan tests/fixtures/negative/protected_write      # exits 0
```

### Supported projects

A directory containing `Cargo.toml`, where at least one `Cargo.toml` in the tree (including workspace members) declares a `soroban-sdk` dependency. All `.rs` files are parsed except under `target`, `node_modules`, `.git`, `vendor`, `dist` and `build`. Scanning only reads files: it never runs `cargo`, build scripts or project code, and makes no network requests.

## Rules

| ID | Name | Status |
|---|---|---|
| SS-001 | Authorization path review | Implemented |
| SS-002 | Arithmetic boundary review | Implemented |
| SS-003 | External contract call review | Planned |
| SS-004 | Error-path review | Planned |
| SS-005 | State-transition test suggestions | Planned |
| SS-006 | Dependency and toolchain diagnostics | Implemented |

**SS-001** reports public `#[contractimpl]` functions that write to contract storage (`.set` / `.remove` on a `.storage()` chain) with no visible `require_auth` / `require_auth_for_args` call in the function or in a same-file helper it calls. It is a review prompt, not a vulnerability claim: authorization enforced in other files, callers or trait methods is invisible to it, and any method named `require_auth` counts as authorization. **SS-002** reports public `#[contractimpl]` functions that apply `+ - *` (or `+= -= *=`) directly to an integer parameter (`i32`/`u32`/`i64`/`u64`/`i128`/`u128`), or cast such a parameter with `as` to a narrower or differently signed type. It is low severity / low confidence: locals, fields and storage values are not tracked, and `overflow-checks` and domain validation are not evaluated.

**SS-006** is informational and reads manifests rather than source. It reports an unpinned `soroban-sdk` (`*`, or a git dependency without `rev`/`tag`), a package depending on `soroban-sdk` without `cdylib` in `[lib] crate-type`, a root manifest without `[profile.release] overflow-checks = true`, and a missing `Cargo.lock` or `rust-toolchain.toml`. It makes no claims about vulnerable dependency versions and consults no advisory data.

Run `sorobanshield rules explain <ID>` for the full list of known false positives and negatives.

## Repository layout

```text
crates/sorobanshield-core    finding model, project discovery, scan engine, reports
crates/sorobanshield-rules   rule implementations and the rule catalog
crates/sorobanshield-cli     the `sorobanshield` binary and its integration tests
tests/fixtures               small Soroban projects used as positive/negative/edge/invalid cases
docs/                        architecture, rule authoring, threat model, limitations, CLI reference
website/                     React + TypeScript documentation site
```

## Develop

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Website (needs Node.js and npm):

```bash
cd website && npm ci && npm run lint && npm run typecheck && npm run test -- --run && npm run build
```

See [CONTRIBUTING.md](CONTRIBUTING.md), [docs/rule-authoring.md](docs/rule-authoring.md) and [SECURITY.md](SECURITY.md). Licensed under the [MIT License](LICENSE).
