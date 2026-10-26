# Threat model

## Assets

The confidentiality of the source code being scanned, the integrity of the machine running the scan, and the trustworthiness of the reported results.

## Trust boundaries

The scanned project is **untrusted input**. SorobanShield treats every file in it as hostile data.

## Mitigations

| Threat | Mitigation |
|---|---|
| Scanning executes attacker code (build scripts, proc-macros, tests) | Never invokes `cargo`, `rustc` or any project code; only reads and parses text with `syn` and `toml` |
| Source exfiltration | No network access and no telemetry. Any future network feature requires a documented privacy review and explicit opt-in |
| Secret leakage in output | Only `.rs` files and `Cargo.toml` are read; `.env` and other files are never opened, and file contents are never echoed except short rule messages |
| Symlink / traversal tricks | Directory walk does not follow symlinks; reported paths are relative to the scan root |
| Pathological input | Parse failures are reported as errors (exit 3), not panics. Resource limits on huge files or deeply nested syntax are **not implemented**; treat very large untrusted inputs with care |
| False sense of security | Every report carries a disclaimer; a scan with unparseable files is reported as incomplete, never clean |

## Out of scope

Compiled WASM, runtime behavior, economic attacks, and anything outside the parsed Rust source.
