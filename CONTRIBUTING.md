# Contributing

Thanks for helping. Keep changes small and focused, and open an issue first for anything that changes rule behavior, the report schema or exit codes.

## Setup

```bash
cargo build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

For the website: `cd website && npm ci && npm run lint && npm run typecheck && npm run test -- --run && npm run build`.

## Pull request checklist

- [ ] Clear purpose; focused diff.
- [ ] Tests cover expected behavior and edge cases (fixtures for rules; CLI tests for user-visible behavior).
- [ ] Docs and `CHANGELOG.md` updated when CLI, report or rule behavior changes.
- [ ] Findings do not overstate certainty; limitations are recorded.
- [ ] No secrets or private source code added.
- [ ] fmt, clippy and tests pass locally.

## Adding or changing a rule

Follow [docs/rule-authoring.md](docs/rule-authoring.md). In short: a stable ID, syntax-aware detection, positive and negative fixtures, edge cases, documented false positives and negatives, and a catalog entry. A rule is only labelled `implemented` once all of that exists and it is registered in `active_rules()`.

## Documentation

Edit the Markdown in `docs/`; the website renders those files directly, so there is no second copy to update. Link between documents as `other-doc.md`, and to repository files as `../path`.

## Releases

See [docs/releasing.md](docs/releasing.md).

## Conduct and security

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). Be respectful and assume good faith. Report vulnerabilities in SorobanShield privately; see [SECURITY.md](SECURITY.md).
