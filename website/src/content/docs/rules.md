# Rule authoring

Rules live in the `sorobanshield-rules` crate. Each rule implements the `Rule` trait from `sorobanshield-core`: a `meta()` method describing the rule and a `check()` method that receives a parsed Rust file and returns findings.

## Requirements for an implemented rule

1. A stable ID (`SS-###`). IDs are never reused.
2. Metadata: concern, detection method, severity and confidence with a rationale, remediation, known false positives and false negatives, references, and fixtures.
3. Syntax-aware detection using `syn`. Avoid regular expressions for decisions that depend on syntax or control flow.
4. A positive fixture that must produce a finding, a negative fixture that must not, and edge cases.
5. Unit tests in the rule module and CLI integration coverage.
6. Registration in `active_rules()`. Only registered rules are run.

## Workflow

```bash
# 1. Add fixtures under tests/fixtures/{positive,negative,edge}/<name>/
#    (each is a tiny project with a Cargo.toml declaring soroban-sdk)
# 2. Implement the rule in crates/sorobanshield-rules/src/
# 3. Register it in active_rules() and remove it from planned.rs
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

## Website sync

The rule catalog on this site is generated from the CLI, so it cannot drift from the code:

```bash
cd website
npm run gen:rules
```

A rule appears as **implemented** only after it is registered and tested. Planned rules are catalog entries with no severity or confidence.

## Severity and confidence

Severity is potential impact. Confidence is how likely the pattern is to be relevant. A pattern that deserves review is not automatically a vulnerability, so prefer low confidence and clear limitations over overstated claims.
