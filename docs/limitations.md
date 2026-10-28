# Limitations

SorobanShield is a heuristic review aid. It can miss real problems and can flag safe code.

## General

- Analysis is per file and syntactic. Macros are not expanded, types are not resolved, and there is no control-flow or cross-file analysis.
- Only Rust source and `Cargo.toml` are examined, not compiled WASM.
- A clean scan means only that the implemented checks reported nothing.
- Six rules (SS-001 to SS-006) are implemented. SS-004, SS-005 and SS-006 are informational prompts about error design, tests and project metadata, not security-defect detectors.

## SS-001 false positives

- Intentionally open public functions, such as one-time initialization.
- Authorization enforced in another file or crate, in a caller, or through a trait method.
- Writes to temporary storage with no security meaning.

## SS-001 false negatives

- `require_auth` present but on the wrong address, or skipped on some paths (any occurrence in the function counts).
- Any method named `require_auth` counts, even on an unrelated type.
- Storage writes via macros, helpers in other files, or APIs other than `set`/`remove`.
- Non-`pub` functions, code outside `#[contractimpl]`, and generated code.
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
