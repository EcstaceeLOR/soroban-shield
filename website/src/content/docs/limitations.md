# Limitations

Static analysis cannot prove that a contract is secure. SorobanShield helps point reviewers at code that deserves a closer look, and it will miss things.

## What a clean scan means

Only that the implemented checks, under the conditions of that scan, reported nothing. Today there is **one** implemented rule ([SS-001](/rules/SS-001)). Authorization bugs outside its pattern, arithmetic issues, cross-contract problems, logic errors and economic attacks are all out of scope.

## False positives

Findings are review prompts. SS-001 can report functions that are intentionally unauthenticated (for example one-time initialization), or whose authorization is enforced in another file, a caller or a trait method.

## False negatives

SS-001 can miss, among other cases:

- a `require_auth` call on the wrong address, or one skipped on some paths;
- storage writes behind macros, in other files, or through APIs other than `set` and `remove`;
- functions that are not `pub` or not inside a `#[contractimpl]` block;
- generated code.

It treats any call named `require_auth` or `require_auth_for_args` as authorization, whatever the receiver is. See [the rule page](/rules/SS-001) for the full list.

## Scope of analysis

- Analysis is per file and syntactic: no type resolution, no macro expansion, no cross-file or control-flow analysis. Same-file helper functions that call `require_auth` are recognised by name only.
- Only Rust source is analysed. Compiled WASM is not.
- Items under `#[cfg(test)]` are skipped.

## Use it alongside

Unit and property tests, manual review, threat modelling and, for contracts that hold real value, an independent audit.
