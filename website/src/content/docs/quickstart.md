# Quickstart

This walks through a first scan using a fixture project that ships in the repository. You need a [built binary](/docs/installation).

## 1. Scan a fixture that should produce a finding

From the repository root:

```bash
./target/release/sorobanshield scan tests/fixtures/positive/unprotected_write
```

The fixture contains a public contract function that writes to storage with no `require_auth` call. The scan reports one SS-001 finding at `src/lib.rs:10` and exits with code `1`. The landing page shows this exact output.

## 2. Scan a fixture that should be clean

```bash
./target/release/sorobanshield scan tests/fixtures/negative/protected_write
echo $?
```

The output says `No findings reported by the implemented checks.` and the exit code is `0`. That means only that the implemented checks found nothing. It is **not** evidence that the contract is secure.

## 3. Get machine-readable output

```bash
./target/release/sorobanshield scan tests/fixtures/positive/unprotected_write --format json
```

The JSON format is described in [Reports](/docs/reports).

## 4. Scan your own contract

```bash
./target/release/sorobanshield scan path/to/your/contract
```

Treat each finding as a prompt to review the code, then confirm with tests and a human review. Read the [limitations](/docs/limitations) first.

## 5. Learn about a rule

```bash
./target/release/sorobanshield rules list
./target/release/sorobanshield rules explain SS-001
```
