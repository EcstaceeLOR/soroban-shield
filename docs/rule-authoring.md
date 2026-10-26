# Rule authoring

A rule implements `sorobanshield_core::Rule`:

```rust
pub trait Rule: Sync {
    fn meta(&self) -> RuleMeta;
    fn check(&self, unit: &SourceUnit<'_>) -> Vec<Finding>;
}
```

`SourceUnit` carries the project-relative path and the parsed `syn::File`. Rules are pure functions of the syntax tree: no I/O, no process execution, no network.

## Workflow

1. **Specify first.** Write down the concern, exactly which syntax you match, what you deliberately do not match, and why severity and confidence are what they are. Prefer one reliable rule over several weak ones. If you cannot detect a problem reliably, emit a low-confidence review prompt rather than a vulnerability claim.
2. **Implement** in `crates/sorobanshield-rules/src/<rule>.rs` using `syn::visit::Visit`. Do not use regular expressions on source text for decisions that need syntax or context. Skip `#[cfg(test)]` items.
3. **Fill in `RuleMeta`**: `concern`, `detection`, `severity_rationale`, `remediation`, `false_positives`, `false_negatives`, `references`, `fixtures`. The website catalog is generated from this metadata (`sorobanshield rules list --format json`), so it cannot drift.
4. **Register** the rule in `active_rules()` in `crates/sorobanshield-rules/src/lib.rs`. A unit test asserts that exactly the `Implemented` rules are active and that planned rules carry no severity.
5. **Add fixtures** under `tests/fixtures/`.
6. **Add tests**: unit tests next to the rule, plus a CLI test in `crates/sorobanshield-cli/tests/cli.rs`.
7. **Document**: update the rule table in `README.md` and `CHANGELOG.md`.

## Fixture conventions

Each fixture is a tiny Soroban project (a `Cargo.toml` declaring `soroban-sdk` and `src/lib.rs`). Fixtures are never compiled; they only need to parse.

| Directory | Meaning |
|---|---|
| `positive/<name>` | Must produce a finding for the rule |
| `negative/<name>` | Must produce no finding (also holds non-Soroban projects used for error tests) |
| `edge/<name>` | Ambiguous or tricky shapes with documented expected behavior |
| `invalid/<name>` | Malformed manifests or source; the scan must fail with exit 3 |

## IDs and status

Rule IDs are stable and never reused. Statuses: `planned` (catalog only, never run), `experimental`, `implemented`, `deprecated`.
