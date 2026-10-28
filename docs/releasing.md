# Releasing

For maintainers. No release has been published yet, and the release workflow has **not been run against a real tag**, so expect to fix small problems on the first attempt. The packaging script is tested locally; the GitHub Actions wiring is not.

## What the workflow does

`.github/workflows/release.yml` runs only when a tag of the form `vMAJOR.MINOR.PATCH` is pushed. Merging to `main` publishes nothing.

1. **verify**: the tag must equal the `sorobanshield` crate version, `CHANGELOG.md` must contain a `## <version>` section, and fmt, clippy and the Rust tests must pass.
2. **build**: `cargo build --release --locked` for Linux (x86_64), macOS (aarch64) and Windows (x86_64), packaged by `scripts/package-release.sh` as `.tar.gz` (or `.zip` on Windows) with a `.sha256` file. The archive holds the binary, `LICENSE`, `README.md` and `CHANGELOG.md`.
3. **publish**: creates a **draft** GitHub release with those files. Nothing becomes public until you publish the draft.

Binaries are not signed, and there is no macOS x86_64 build.

## Cutting a release

1. Move the `## Unreleased` entries in `CHANGELOG.md` under a new `## 0.1.0` heading (keep an empty `## Unreleased` above it).
2. Set the version in the root `Cargo.toml` (`[workspace.package]`) and refresh `Cargo.lock`.
3. Confirm the checks in [CONTRIBUTING.md](../CONTRIBUTING.md) pass, and that README and docs still describe only what exists. Update the "no release" statements in `README.md`, `docs/installation.md` and the website once binaries exist.
4. Commit, then tag and push: `git tag v0.1.0 && git push origin v0.1.0`.
5. Check the draft release: archive names, checksums (`sha256sum -c`), and that the binary runs (`sorobanshield --version`). Publish the draft.

Test the packaging locally with `scripts/package-release.sh <version> <target-name> <path-to-binary>`; output goes to `dist/`.
