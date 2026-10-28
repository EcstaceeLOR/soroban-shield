#!/usr/bin/env bash
# Runs the built CLI over every fixture project, writes SARIF, and validates each
# log against the official OASIS SARIF 2.1.0 schema. Needs: a release or debug
# build of sorobanshield, node, curl and network access to fetch the schema.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
bin="${SOROBANSHIELD_BIN:-$root/target/debug/sorobanshield}"
schema_url="https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json"
schema_sha256="c3b4bb2d6093897483348925aaa73af03b3e3f4bd4ca38cef26dcb4212a2682e"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

curl -fsSL "$schema_url" -o "$work/schema.json"
echo "$schema_sha256  $work/schema.json" | sha256sum -c --status || {
  echo "SARIF schema changed upstream (sha256 mismatch); review it and update scripts/check-sarif.sh" >&2
  exit 1
}

(cd "$root/scripts/validate-sarif" && npm ci --silent)

logs=()
for dir in "$root"/tests/fixtures/{positive,negative,edge,invalid}/*/; do
  name="$(basename "$(dirname "$dir")")-$(basename "$dir")"
  # Exit 0 (clean), 1 (findings) and 3 (scan incomplete) can all write a log.
  # When the project cannot be discovered at all (exit 2 or 3) nothing is
  # written to stdout, by design, so there is nothing to validate.
  set +e
  "$bin" scan "$dir" --format sarif > "$work/$name.sarif" 2>/dev/null
  rc=$?
  set -e
  if [ "$rc" -gt 3 ]; then
    echo "unexpected exit code $rc for $dir" >&2
    exit 1
  fi
  if [ -s "$work/$name.sarif" ]; then
    logs+=("$work/$name.sarif")
  else
    echo "skip  $name (exit $rc, no log written)"
  fi
done

node "$root/scripts/validate-sarif/validate.mjs" "$work/schema.json" "${logs[@]}"
