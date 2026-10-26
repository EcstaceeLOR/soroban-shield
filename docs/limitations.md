# Limitations

SorobanShield is a heuristic review aid. It can miss real problems and can flag safe code.

## General

- Analysis is per file and syntactic. Macros are not expanded, types are not resolved, and there is no control-flow or cross-file analysis.
- Only Rust source and `Cargo.toml` are examined, not compiled WASM.
- A clean scan means only that the implemented checks reported nothing.
- Only one rule (SS-001) is implemented. SS-002 to SS-006 are planned and do not run.

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

## Other

- No configuration file or rule exclusions yet; there is no baseline support.
- No limits on input size are enforced.
- Tested only on Linux with Rust 1.99.
