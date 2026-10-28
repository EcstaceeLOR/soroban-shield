# Limitations

Static analysis cannot prove that a contract is secure. SorobanShield is a heuristic review aid: it can miss real problems and can flag safe code.

## General

- Analysis is syntactic. Macros are not expanded, types are not resolved, and there is no control-flow analysis. Only SS-001 and SS-005 look across files, and SS-001 does so by function name alone (see below).
- Only Rust source and `Cargo.toml` are examined, not compiled WASM.
- Test code is recognised by `#[cfg(test)]` (inline modules, `#![cfg(test)]` files, files under a `tests/` directory (Cargo integration tests), and `#[cfg(test)] mod name;` files found by the standard `name.rs` / `name/mod.rs` layout). `#[path = ".."]` is not followed, so such a test file is analysed like normal source. Test-only files are not reported on, but SS-005 reads them as test code.
- `__constructor` is skipped by SS-001 and SS-005 because it runs only at deployment. A constructor that should enforce a deployer check is therefore not examined.
- A clean scan means only that the implemented checks reported nothing.
- Six rules (SS-001 to SS-006) are implemented. SS-004, SS-005 and SS-006 are informational prompts about error design, tests and project metadata, not security-defect detectors.

## SS-001 false positives

- Intentionally open public functions: one-time initialization guarded by an "already initialized" check, and permissionless keeper-style functions anyone may call by design (for example distributing emissions or opening an auction).
- Authorization enforced by a Merkle proof, a signature or another scheme that is not `require_auth`.
- Authorization enforced in another crate, in a caller, or through a trait method.
- Writes to temporary storage with no security meaning.
- A write attributed through a helper that is not the function actually called: calls are matched by name only.

## SS-001 false negatives

- `require_auth` present but on the wrong address, or skipped on some paths (any occurrence in the function counts).
- Any method named `require_auth` counts, even on an unrelated type, and so does any attribute macro named `only_owner`, `only_admin`, `only_role` or `only_any_role`, whatever it expands to. `has_role` and `has_any_role` are not counted because they check membership without `require_auth`.
- Calls are followed by name across files, but only when exactly one function of that name exists in the project, and for writes only through path calls (`storage::set_x(..)`, `Self::f(..)`), not method calls (`x.save(..)`). Names defined more than once, other crates, and macros are not followed.
- Storage APIs other than `set`/`remove`.
- Code outside `#[contractimpl]` and generated code.
- Items under `#[cfg(test)]` are intentionally skipped.

## SS-002 false positives

- Arithmetic on values already bounded by earlier validation or by the domain.
- Builds with `overflow-checks` enabled, where overflow traps deterministically and that is acceptable.
- Intentional wrapping or truncating behavior.

## SS-002 false negatives

- Arithmetic on locals, struct fields, storage reads or function results: only direct integer-parameter operands are tracked.
- Operations in private helpers, other files or macros.
- Type aliases, and `i8`/`i16`/`u8`/`u16`/`usize`/`isize` parameters.
- Division, remainder, shifts, casts of non-parameter expressions, and fixed-point rounding issues.

## SS-003 false positives

- Routers and proxies that intentionally call a caller-chosen contract; addresses validated earlier in the function (validation is not evaluated); best-effort calls whose failure is acceptable.

## SS-003 false negatives

- Targets reached through locals, struct fields, storage reads or helpers (only direct `Address` parameters and same-function client bindings are tracked); `try_*` calls on clients obtained other than by `...Client::new(..)` in the same function; calls in private helpers, other files or macros; ordering of state writes relative to external calls.

## SS-004 false positives and negatives

- `unwrap`/`expect` on values guaranteed present by construction are reported. Panics in private helpers, other files or macro arguments, `unreachable!`, indexing and division by zero are not reported. A failed Soroban invocation reverts its state changes, so this rule is about error design and diagnosability, not partial state.

## SS-005 false positives and negatives

- Matching is by function name. Tests in other crates are not seen (false positive); any identifier with the same name in test code counts as coverage even if the transition is not meaningfully tested (false negative); sequences of transitions are never inspected.

## SS-006 false positives

- `overflow-checks` set through `.cargo/config.toml` or `CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS`, which are not read.
- Helper crates that depend on `soroban-sdk` but are not contracts (no `cdylib` needed).
- Toolchain pinned by CI or a parent directory; `Cargo.lock` deliberately omitted.

## SS-006 false negatives

- Vulnerable or outdated dependency versions: no advisory database is consulted, and the rule never claims a version is vulnerable.
- Inconsistent `soroban-sdk` versions across workspace members, or a toolchain that does not match the SDK.
- Settings inherited through `[workspace.package]` or profile `inherits` chains.

## Other

- Configuration supports path-prefix exclusions and disabling whole rules only. There are no globs. Inline `sorobanshield-ignore` comments hide findings on Rust source lines (see the CLI reference); manifest and project-wide findings can only be disabled by rule or baselined. Baselines can hide recorded findings but say nothing about whether they were reviewed; a baseline entry matches on rule, file and message, so a message change (for example a renamed function) resurfaces the finding.
- No limits on input size are enforced.
- Tested only on Linux with Rust 1.99.

## Use it alongside

Unit and property tests, manual review, threat modelling and, for contracts that hold real value, an independent audit.

## Trial on real code

The scanner was run on [stellar/soroban-examples](https://github.com/stellar/soroban-examples) at commit `03d42aa` (32 independent projects, 117 Rust files). Every file parsed without error. Before the changes recorded in the changelog for `__constructor` and out-of-line test modules, the scan reported 114 findings (SS-001 19, SS-002 8, SS-003 9, SS-004 28, SS-005 10, SS-006 40); afterwards it reported 95 (SS-001 10, SS-002 6, SS-003 9, SS-004 26, SS-005 4, SS-006 40). Most remaining SS-001 findings are tutorial counters whose `increment` is intentionally open, which is the false-positive class listed above.

Three larger codebases were scanned afterwards, with no parse errors. These counts are after SS-001 learned to follow storage writes and authorization across files, to read OpenZeppelin access-control attributes, and to treat the methods of `#[contractimpl] impl Trait for Type` blocks as entrypoints (they carry no `pub` keyword, so they had been invisible to every rule):

| Codebase (commit) | Rust files | Findings |
|---|---|---|
| [OpenZeppelin/stellar-contracts](https://github.com/OpenZeppelin/stellar-contracts) (`ddc327e`) | 376 | 35: SS-004 19, SS-001 8, SS-005 4, SS-003 3, SS-006 1 |
| [blend-capital/blend-contracts](https://github.com/blend-capital/blend-contracts) (`895845f`) | 88 | 16: SS-001 13, SS-003 1, SS-006 2 |
| [soroswap/core](https://github.com/soroswap/core) (`6eade00`, 6 contract crates) | 74 | 27: SS-006 10, SS-001 8, SS-004 7, SS-003 2 |

Before those three changes the same scan of Blend reported only two SS-006 notes, because its entrypoints are trait-impl methods that delegate every storage write to another file. That near-silence was a blind spot, not evidence of safety. Most of Blend's 13 SS-001 findings are permissionless functions that anyone may call by design, which is what a low-confidence review prompt is expected to surface. In the OpenZeppelin examples, `vote` in `merkle-voting` is reported although access is gated by a Merkle proof rather than `require_auth`, which the rule cannot know. Scanning this corpus also showed that files under a `tests/` directory were being analysed as contract code, which is now fixed.

This is a check that the scanner handles real syntax and a source of false-positive classes. It is **not** a measurement of accuracy: these are teaching examples, not audited production contracts, nobody triaged every finding for true positives, and the same projects informed the two fixes.
