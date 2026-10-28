# Installation

SorobanShield is **not yet published**. There is no crates.io package, no prebuilt binary and no tagged release. The only supported way to run it today is to build it from source.

## Prerequisites

- Git
- A recent stable Rust toolchain installed with [rustup](https://rustup.rs). The workspace uses the 2021 edition.

## Build from source

```bash
git clone https://github.com/Sorobanshield/soroban-shield.git
cd soroban-shield
cargo build --release
```

The binary is written to `target/release/sorobanshield`. Check that it runs:

```bash
./target/release/sorobanshield --version
./target/release/sorobanshield --help
```

Optionally put it on your `PATH`, or run it in place as shown in the [quickstart](quickstart.md).

## Run the tests

```bash
cargo test --workspace
```

## Supported project layout

A directory is scannable when:

- it contains a `Cargo.toml` at the path you pass to `scan`;
- at least one `Cargo.toml` under it declares a `soroban-sdk` dependency (in `[dependencies]`, `[dev-dependencies]` or `[workspace.dependencies]`);
- it contains at least one `.rs` file outside the skipped directories.

The directories `target`, `node_modules`, `.git`, `vendor`, `dist` and `build` are never scanned. Anything else is reported as an error, not as a clean scan. See the [CLI reference](cli-reference.md) for exit codes.
