# Security Policy

## Reporting a vulnerability in SorobanShield

Please report vulnerabilities privately through GitHub's private vulnerability reporting:
<https://github.com/sorobanshield/soroban-shield/security/advisories/new>

Do not open a public issue for a suspected vulnerability. Include the affected version or commit, a minimal reproduction, and the impact you expect. Maintainers will acknowledge reports on a best-effort basis; this is a volunteer project and no response-time guarantee is made.

In scope: the CLI and core crates (for example crashes or unbounded resource use on crafted input, path-traversal or file-write behavior, anything that executes project code or sends data off the machine) and the website.

Out of scope: false positives and false negatives in rules. Those are normal bugs; open a regular issue (see `docs/limitations.md`).

## Vulnerabilities in other projects you find with this tool

Do not publish exploitable details about a third-party project before coordinating disclosure with its maintainers.

## Supported versions

No release has been published yet. Only the `main` branch is supported.

## Design commitments

SorobanShield scans locally, does not execute project code or build scripts, does not make network requests, and does not collect telemetry. See `docs/threat-model.md`.
