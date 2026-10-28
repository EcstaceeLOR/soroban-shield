# Installation

There is no published release, crate or prebuilt binary. Build from source.

Prerequisites: git and a stable Rust toolchain installed with [rustup](https://rustup.rs). Development has used Rust 1.99 on Linux; other versions and platforms are untested.

```bash
git clone https://github.com/Sorobanshield/soroban-shield.git
cd soroban-shield
cargo build --release
./target/release/sorobanshield --version
```

Optionally install it onto your `PATH`:

```bash
cargo install --path crates/sorobanshield-cli
```
