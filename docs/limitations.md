# Limitations

SorobanShield is a heuristic review aid. It can miss real problems and can flag safe code.

## General

- Analysis is per file and syntactic. Macros are not expanded, types are not resolved, and there is no control-flow or cross-file analysis.
- Only Rust source and `Cargo.toml` are examined, not compiled WASM.
- A clean scan means only that the implemented checks reported nothing.
- Three rules (SS-001, SS-002, SS-006) are implemented. SS-003 to SS-005 are planned and do not run.

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

## SS-006 false positives

- `overflow-checks` set through `.cargo/config.toml` or `CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS`, which are not read.
- Helper crates that depend on `soroban-sdk` but are not contracts (no `cdylib` needed).
- Toolchain pinned by CI or a parent directory; `Cargo.lock` deliberately omitted.

## SS-006 false negatives

- Vulnerable or outdated dependency versions: no advisory database is consulted, and the rule never claims a version is vulnerable.
- Inconsistent `soroban-sdk` versions across workspace members, or a toolchain that does not match the SDK.
- Settings inherited through `[workspace.package]` or profile `inherits` chains.

## Other

- Configuration supports path-prefix exclusions and disabling whole rules only. There are no globs, no per-finding suppressions and no baseline support.
- No limits on input size are enforced.
- Tested only on Linux with Rust 1.99.
