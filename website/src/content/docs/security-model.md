# Security model

SorobanShield is a security-related tool, so its own behaviour is deliberately constrained.

## What it does with your code

- Runs locally. There is no telemetry and no network access in the scanner.
- Reads `Cargo.toml` and `.rs` files only, and parses them as text. It does **not** compile your project, run build scripts, or execute any project code.
- Never writes to your project.
- Does not read `.env` files or print environment variables.
- Does not follow symbolic links while walking the project.

## Trust boundaries

Source files and manifests are untrusted input. Malformed input produces an error report and exit code `3` instead of a clean result. The scanner depends on the `syn` parser; a bug in a dependency is a possible risk like in any software.

## What it does not do

- It does not prove a contract is secure, and a clean scan is not a safety claim.
- It has not been independently audited. No security review of SorobanShield itself has been performed.
- Remote scanning or hosted dashboards do not exist. Any future feature that handles source code remotely needs a documented threat model and privacy review before release.

## Reporting a vulnerability

If you find a vulnerability in SorobanShield itself, do not open a public issue with exploit details. Follow the process in [SECURITY.md](https://github.com/sorobanshield/soroban-shield/blob/main/SECURITY.md) in the repository.
