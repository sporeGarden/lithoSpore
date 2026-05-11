#!/bin/bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Build the LTEE guideStone artifact — cross-compile ecoBin binaries
# and assemble the artifact directory for USB deployment.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(dirname "$SCRIPT_DIR")"
ARTIFACT="$ROOT/artifact"
VERSION=$(grep '^version' "$ROOT/Cargo.toml" | head -1 | sed 's/.*= *"\(.*\)"/\1/' || echo "0.1.0")

echo "Building lithoSpore LTEE guideStone artifact v$VERSION"
echo ""

# --- x86_64 musl-static ---
echo "=== x86_64-unknown-linux-musl ==="
if rustup target list --installed | grep -q x86_64-unknown-linux-musl; then
    cargo build --release --target x86_64-unknown-linux-musl
    for bin in ltee-fitness ltee-mutations ltee-alleles ltee-citrate ltee-biobricks ltee-breseq ltee-anderson litho; do
        cp "$ROOT/target/x86_64-unknown-linux-musl/release/$bin" "$ARTIFACT/bin/x86_64/static/" 2>/dev/null || true
    done
    echo "  Binaries copied to artifact/bin/x86_64/static/"
else
    echo "  SKIP: x86_64-unknown-linux-musl target not installed"
    echo "  Install with: rustup target add x86_64-unknown-linux-musl"
fi

# --- aarch64 musl-static ---
echo ""
echo "=== aarch64-unknown-linux-musl ==="
if rustup target list --installed | grep -q aarch64-unknown-linux-musl; then
    cargo build --release --target aarch64-unknown-linux-musl
    for bin in ltee-fitness ltee-mutations ltee-alleles ltee-citrate ltee-biobricks ltee-breseq ltee-anderson litho; do
        cp "$ROOT/target/aarch64-unknown-linux-musl/release/$bin" "$ARTIFACT/bin/aarch64/static/" 2>/dev/null || true
    done
    echo "  Binaries copied to artifact/bin/aarch64/static/"
else
    echo "  SKIP: aarch64-unknown-linux-musl target not installed"
    echo "  Install with: rustup target add aarch64-unknown-linux-musl"
fi

# --- Generate CHECKSUMS ---
echo ""
echo "=== Generating CHECKSUMS ==="
cd "$ARTIFACT"
if command -v b3sum >/dev/null 2>&1; then
    find . -type f ! -name CHECKSUMS ! -path './liveSpore.json' | sort | xargs b3sum > CHECKSUMS
    echo "  CHECKSUMS generated ($(wc -l < CHECKSUMS) files)"
else
    echo "  SKIP: b3sum not installed. Install with: cargo install b3sum"
fi

echo ""
echo "Artifact ready at: $ARTIFACT"
echo "Total size: $(du -sh "$ARTIFACT" | cut -f1)"
