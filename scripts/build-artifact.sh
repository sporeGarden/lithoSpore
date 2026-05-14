#!/bin/bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Build the LTEE guideStone artifact — cross-compile ecoBin binaries
# and assemble the artifact directory for USB deployment.
#
# Usage:
#   ./scripts/build-artifact.sh              # Build to artifact/bin/{arch}/static/
#   ./scripts/build-artifact.sh --flat DIR   # Build to flat DIR/bin/ (USB layout)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(dirname "$SCRIPT_DIR")"
ARTIFACT="$ROOT/artifact"
VERSION=$(grep '^version' "$ROOT/Cargo.toml" | head -1 | sed 's/.*= *"\(.*\)"/\1/' || echo "0.1.0")
FLAT_DIR=""

while [ $# -gt 0 ]; do
    case "$1" in
        --flat) FLAT_DIR="$2"; shift 2 ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

BINS="ltee-fitness ltee-mutations ltee-alleles ltee-citrate ltee-biobricks ltee-breseq ltee-anderson litho"

echo "Building lithoSpore LTEE guideStone artifact v$VERSION"
echo ""

copy_bins() {
    local target_triple="$1"
    local dest="$2"
    mkdir -p "$dest"
    for bin in $BINS; do
        cp "$ROOT/target/$target_triple/release/$bin" "$dest/" 2>/dev/null || true
    done
    echo "  Binaries copied to $dest"
}

# --- x86_64 musl-static ---
echo "=== x86_64-unknown-linux-musl ==="
if rustup target list --installed | grep -q x86_64-unknown-linux-musl; then
    cargo build --release --target x86_64-unknown-linux-musl
    if [ -n "$FLAT_DIR" ]; then
        copy_bins "x86_64-unknown-linux-musl" "$FLAT_DIR/bin"
    else
        copy_bins "x86_64-unknown-linux-musl" "$ARTIFACT/bin/x86_64/static"
    fi
else
    echo "  SKIP: x86_64-unknown-linux-musl target not installed"
    echo "  Install with: rustup target add x86_64-unknown-linux-musl"
fi

# --- aarch64 musl-static ---
echo ""
echo "=== aarch64-unknown-linux-musl ==="
if rustup target list --installed | grep -q aarch64-unknown-linux-musl; then
    cargo build --release --target aarch64-unknown-linux-musl
    if [ -n "$FLAT_DIR" ]; then
        copy_bins "aarch64-unknown-linux-musl" "$FLAT_DIR/bin"
    else
        copy_bins "aarch64-unknown-linux-musl" "$ARTIFACT/bin/aarch64/static"
    fi
else
    echo "  SKIP: aarch64-unknown-linux-musl target not installed"
    echo "  Install with: rustup target add aarch64-unknown-linux-musl"
fi

# --- Generate CHECKSUMS ---
echo ""
echo "=== Generating CHECKSUMS ==="
CHECKSUM_DIR="${FLAT_DIR:-$ARTIFACT}"
cd "$CHECKSUM_DIR"
if command -v b3sum >/dev/null 2>&1; then
    find . -type f ! -name CHECKSUMS ! -path './liveSpore.json' | sort | xargs b3sum > CHECKSUMS
    echo "  CHECKSUMS generated ($(wc -l < CHECKSUMS) files)"
else
    echo "  SKIP: b3sum not installed. Install with: cargo install b3sum"
fi

echo ""
echo "Artifact ready at: $CHECKSUM_DIR"
echo "Total size: $(du -sh "$CHECKSUM_DIR" | cut -f1)"
