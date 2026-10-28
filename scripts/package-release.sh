#!/usr/bin/env bash
# Packages a built sorobanshield binary into dist/ as an archive plus a SHA-256 file.
#   scripts/package-release.sh <version> <target-name> <path-to-binary>
# e.g. scripts/package-release.sh 0.1.0 x86_64-unknown-linux-gnu target/release/sorobanshield
# Produces .tar.gz (or .zip when the binary ends in .exe).
set -euo pipefail

if [ "$#" -ne 3 ]; then
  echo "usage: $0 <version> <target-name> <path-to-binary>" >&2
  exit 2
fi
version="$1"; target="$2"; binary="$3"
root="$(cd "$(dirname "$0")/.." && pwd)"
[ -f "$binary" ] || { echo "binary not found: $binary" >&2; exit 1; }

name="sorobanshield-${version}-${target}"
stage="$(mktemp -d)"
trap 'rm -rf "$stage"' EXIT
mkdir -p "$stage/$name" "$root/dist"
cp "$binary" "$stage/$name/"
cp "$root/LICENSE" "$root/README.md" "$root/CHANGELOG.md" "$stage/$name/"

if [[ "$binary" == *.exe ]]; then
  archive="$name.zip"
  (cd "$stage" && python3 -c "import shutil,sys; shutil.make_archive(sys.argv[1], 'zip', '.', sys.argv[1])" "$name")
  mv "$stage/$name.zip" "$root/dist/$archive"
else
  archive="$name.tar.gz"
  tar -C "$stage" -czf "$root/dist/$archive" "$name"
fi
(cd "$root/dist" && { sha256sum "$archive" 2>/dev/null || shasum -a 256 "$archive"; } > "$archive.sha256")
echo "dist/$archive"
