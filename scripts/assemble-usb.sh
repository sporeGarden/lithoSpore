#!/bin/bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# assemble-usb.sh — Build the lithoSpore USB (hypogeal cotyledon).
#
# Produces a complete, self-sufficient USB directory matching
# wateringHole/LITHOSPORE_USB_DEPLOYMENT.md.
#
# Usage:
#   ./scripts/assemble-usb.sh                           # Build to ./usb-staging/
#   ./scripts/assemble-usb.sh --target /media/lithoSpore # Build to USB mount
#   ./scripts/assemble-usb.sh --skip-python              # Skip Python embedding
#   ./scripts/assemble-usb.sh --skip-fetch               # Skip data fetching
#   ./scripts/assemble-usb.sh --skip-build               # Skip binary compilation
#   ./scripts/assemble-usb.sh --dry-run                  # Show what would happen

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(dirname "$SCRIPT_DIR")"
TARGET="${TARGET:-$ROOT/usb-staging}"
ARCH="$(uname -m)"
SKIP_PYTHON=false
SKIP_FETCH=false
SKIP_BUILD=false
DRY_RUN=false

while [ $# -gt 0 ]; do
    case "$1" in
        --target)      TARGET="$2"; shift 2 ;;
        --skip-python) SKIP_PYTHON=true; shift ;;
        --skip-fetch)  SKIP_FETCH=true; shift ;;
        --skip-build)  SKIP_BUILD=true; shift ;;
        --dry-run)     DRY_RUN=true; shift ;;
        --help|-h)
            echo "Usage: $0 [--target DIR] [--skip-python] [--skip-fetch] [--skip-build] [--dry-run]"
            exit 0
            ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

case "$ARCH" in
    arm64) ARCH="aarch64" ;;
esac

log() { echo "==> $*"; }
step() { echo ""; echo "=== $* ==="; }

if $DRY_RUN; then
    log "DRY RUN — showing what would be assembled"
    log "  Source:  $ROOT"
    log "  Target:  $TARGET"
    log "  Arch:    $ARCH"
    log "  Python:  $( $SKIP_PYTHON && echo SKIP || echo EMBED )"
    log "  Fetch:   $( $SKIP_FETCH && echo SKIP || echo FETCH )"
    log "  Build:   $( $SKIP_BUILD && echo SKIP || echo BUILD )"
    echo ""
    echo "Directory tree that would be created:"
    echo "  $TARGET/"
    echo "  ├── .biomeos-spore"
    echo "  ├── .family.seed"
    echo "  ├── spore.sh"
    echo "  ├── validate"
    echo "  ├── refresh"
    echo "  ├── liveSpore.json"
    echo "  ├── data_manifest.toml"
    echo "  ├── biomeOS/"
    echo "  │   ├── tower.toml"
    echo "  │   └── graphs/lithoSpore_validation.toml"
    echo "  ├── bin/"
    echo "  │   ├── litho"
    echo "  │   ├── ltee-fitness"
    echo "  │   ├── ltee-mutations"
    echo "  │   ├── ltee-alleles"
    echo "  │   ├── ltee-citrate"
    echo "  │   ├── ltee-biobricks"
    echo "  │   ├── ltee-breseq"
    echo "  │   └── ltee-anderson"
    echo "  ├── python/  (if not --skip-python)"
    echo "  ├── artifact/"
    echo "  │   ├── data/  (7 LTEE datasets)"
    echo "  │   ├── scope.toml"
    echo "  │   ├── data.toml"
    echo "  │   └── tolerances.toml"
    echo "  ├── validation/"
    echo "  │   └── expected/  (module expected-value JSONs)"
    echo "  ├── foundation/"
    echo "  │   └── targets/  (validation target TOMLs)"
    echo "  ├── figures/"
    echo "  │   └── m[1-7]_*.svg  (publication-quality figures)"
    echo "  └── notebooks/"
    echo "      ├── litho_figures.py"
    echo "      ├── module*/*.py"
    echo "      └── *.html  (pre-rendered)"
    exit 0
fi

step "lithoSpore USB Assembly — Hypogeal Cotyledon"
log "Source:  $ROOT"
log "Target:  $TARGET"
log "Arch:    $ARCH"

# --- 1. Create directory tree ---
step "1. Creating directory tree"
mkdir -p "$TARGET"/{bin,artifact/data,validation/expected,foundation/targets,notebooks,figures,biomeOS/graphs}
if ! $SKIP_PYTHON; then
    mkdir -p "$TARGET/python"
fi
log "Directory tree created"

# --- 2. Stage root files ---
step "2. Staging root files"
cp "$ROOT/artifact/usb-root/.biomeos-spore" "$TARGET/"
cp "$ROOT/artifact/usb-root/validate"       "$TARGET/"
cp "$ROOT/artifact/usb-root/refresh"        "$TARGET/"
cp "$ROOT/artifact/usb-root/verify"         "$TARGET/"
cp "$ROOT/artifact/usb-root/spore.sh"       "$TARGET/"
chmod +x "$TARGET/validate" "$TARGET/refresh" "$TARGET/verify" "$TARGET/spore.sh"

touch "$TARGET/.family.seed"

if [ -f "$TARGET/liveSpore.json" ]; then
    log "liveSpore.json already exists — preserving"
else
    echo "[]" > "$TARGET/liveSpore.json"
    log "liveSpore.json initialized (empty)"
fi
log "Root files staged"

# --- 3. Stage biomeOS files ---
step "3. Staging biomeOS files"
cp "$ROOT/artifact/usb-root/biomeOS/tower.toml"                         "$TARGET/biomeOS/"
cp "$ROOT/artifact/usb-root/biomeOS/graphs/lithoSpore_validation.toml"  "$TARGET/biomeOS/graphs/"
log "biomeOS files staged"

# --- 4. Build and stage ecoBin binaries ---
step "4. Building ecoBin binaries"
BINS="ltee-fitness ltee-mutations ltee-alleles ltee-citrate ltee-biobricks ltee-breseq ltee-anderson litho"

if $SKIP_BUILD; then
    log "SKIP: --skip-build specified"
else
    MUSL_TARGET="${ARCH}-unknown-linux-musl"
    if rustup target list --installed 2>/dev/null | grep -q "$MUSL_TARGET"; then
        log "Building for $MUSL_TARGET..."
        cargo build --release --target "$MUSL_TARGET" --manifest-path "$ROOT/Cargo.toml"
        for bin in $BINS; do
            src="$ROOT/target/$MUSL_TARGET/release/$bin"
            if [ -f "$src" ]; then
                cp "$src" "$TARGET/bin/"
                log "  $bin ($(du -h "$src" | cut -f1) musl-static)"
            fi
        done
    else
        log "musl target $MUSL_TARGET not installed, trying native release..."
        cargo build --release --manifest-path "$ROOT/Cargo.toml"
        for bin in $BINS; do
            src="$ROOT/target/release/$bin"
            if [ -f "$src" ]; then
                cp "$src" "$TARGET/bin/"
                log "  $bin ($(du -h "$src" | cut -f1) native)"
            fi
        done
        log "WARNING: native binaries are NOT musl-static — USB portability limited"
    fi
fi

STAGED_BINS=$(find "$TARGET/bin" -type f 2>/dev/null | wc -l)
log "$STAGED_BINS binaries staged"

# --- 5. Fetch and stage data bundles ---
step "5. Staging data bundles"
if $SKIP_FETCH; then
    log "SKIP: --skip-fetch specified"
else
    for script in "$ROOT"/scripts/fetch_*.sh; do
        [ -f "$script" ] || continue
        log "  Running $(basename "$script")..."
        bash "$script" || log "  WARNING: $(basename "$script") failed (non-fatal)"
    done
fi

for ds_dir in "$ROOT"/artifact/data/*/; do
    [ -d "$ds_dir" ] || continue
    ds_name="$(basename "$ds_dir")"
    if [ -n "$(ls -A "$ds_dir" 2>/dev/null)" ]; then
        cp -r "$ds_dir" "$TARGET/artifact/data/"
        log "  Data bundle: $ds_name"
    fi
done

cp "$ROOT/artifact/scope.toml"      "$TARGET/artifact/"
cp "$ROOT/artifact/data.toml"       "$TARGET/artifact/"
cp "$ROOT/artifact/tolerances.toml" "$TARGET/artifact/"
log "Data bundles staged"

# --- 6. Stage Python runtime ---
step "6. Staging Python runtime"
if $SKIP_PYTHON; then
    log "SKIP: --skip-python specified"
elif [ -d "$TARGET/python/bin" ] && [ -f "$TARGET/python/bin/python3" ]; then
    log "Python runtime already staged — preserving"
else
    PYTHON_STANDALONE_URL="https://github.com/indygreg/python-build-standalone/releases/download/20240415/cpython-3.12.3+20240415-${ARCH}-unknown-linux-gnu-install_only.tar.gz"
    log "Downloading python-build-standalone for $ARCH..."
    if curl -fsSL "$PYTHON_STANDALONE_URL" -o "/tmp/python-standalone.tar.gz" 2>/dev/null; then
        tar xzf "/tmp/python-standalone.tar.gz" -C "$TARGET/"
        rm -f "/tmp/python-standalone.tar.gz"
        log "Python runtime embedded"

        log "Installing numpy + scipy + matplotlib into embedded Python..."
        "$TARGET/python/bin/python3" -m pip install --quiet numpy scipy matplotlib 2>/dev/null || \
            log "  WARNING: pip install failed — Tier 1 may have limited functionality"
    else
        log "WARNING: python-build-standalone download failed"
        log "  Tier 1 will use system python3 if available"
    fi
fi

# --- 7. Stage notebooks ---
step "7. Staging notebooks"
for mod_dir in "$ROOT"/notebooks/module*/; do
    [ -d "$mod_dir" ] || continue
    mod_name="$(basename "$mod_dir")"
    mkdir -p "$TARGET/notebooks/$mod_name"
    cp "$mod_dir"/*.py "$TARGET/notebooks/$mod_name/" 2>/dev/null || true
    log "  $mod_name"
done

if command -v jupyter >/dev/null 2>&1 || command -v jupyter-nbconvert >/dev/null 2>&1; then
    log "Rendering HTML notebooks..."
    for py in "$TARGET"/notebooks/module*/*.py; do
        [ -f "$py" ] || continue
        html="${py%.py}.html"
        jupyter nbconvert --to html "$py" --output "$html" 2>/dev/null || true
    done
else
    log "  jupyter-nbconvert not found — HTML rendering skipped"
    log "  Pre-rendered HTML can be added manually to notebooks/"
fi

if [ -d "$ROOT/artifact/notebooks/html" ]; then
    cp "$ROOT"/artifact/notebooks/html/*.html "$TARGET/notebooks/" 2>/dev/null || true
fi

if [ -f "$ROOT/notebooks/litho_figures.py" ]; then
    cp "$ROOT/notebooks/litho_figures.py" "$TARGET/notebooks/"
    log "  litho_figures.py helper staged"
fi
log "Notebooks staged"

# --- 7b. Generate and stage figures ---
step "7b. Generating scientific figures"
mkdir -p "$TARGET/figures"
if ! $SKIP_PYTHON; then
    PYBIN="$TARGET/python/bin/python3"
    [ -x "$PYBIN" ] || PYBIN="$(command -v python3 2>/dev/null || true)"

    if [ -n "$PYBIN" ] && [ -x "$PYBIN" ]; then
        for mod_py in "$ROOT"/notebooks/module*/*.py; do
            [ -f "$mod_py" ] || continue
            mod_name="$(basename "$(dirname "$mod_py")")"
            log "  Generating figures for $mod_name..."
            PYTHONPATH="$ROOT/notebooks" "$PYBIN" "$mod_py" >/dev/null 2>&1 || \
                log "    WARNING: figure generation failed for $mod_name"
        done
    fi
fi

if [ -d "$ROOT/figures" ]; then
    cp "$ROOT"/figures/*.svg "$TARGET/figures/" 2>/dev/null || true
    FIG_COUNT=$(find "$TARGET/figures" -name "*.svg" 2>/dev/null | wc -l)
    log "$FIG_COUNT SVG figures staged"
else
    log "  No figures generated — skipping"
fi

# --- 8. Stage expected values ---
step "8. Staging expected values"
if [ -d "$ROOT/validation/expected" ]; then
    cp "$ROOT"/validation/expected/*.json "$TARGET/validation/expected/" 2>/dev/null || true
    EXPECTED_COUNT=$(find "$TARGET/validation/expected" -name "*.json" 2>/dev/null | wc -l)
    log "$EXPECTED_COUNT expected-value JSONs staged"
else
    log "  No validation/expected/ directory found — skipping"
fi

# --- 9. Stage Foundation targets ---
step "9. Staging Foundation targets"
if [ -d "$ROOT/data/targets" ]; then
    cp "$ROOT"/data/targets/*.toml "$TARGET/foundation/targets/" 2>/dev/null || true
    TARGETS_COUNT=$(find "$TARGET/foundation/targets" -name "*.toml" 2>/dev/null | wc -l)
    log "$TARGETS_COUNT Foundation target TOMLs staged"
else
    log "  No data/targets/ directory found — skipping"
fi

# --- 10. Generate data_manifest.toml ---
step "10. Generating data_manifest.toml"
if command -v b3sum >/dev/null 2>&1; then
    MANIFEST="$TARGET/data_manifest.toml"
    {
        echo "# SPDX-License-Identifier: AGPL-3.0-or-later"
        echo "#"
        echo "# Data manifest — BLAKE3 inventory of all bundled data."
        echo "# Generated by assemble-usb.sh on $(date -u +%Y-%m-%dT%H:%M:%SZ)"
        echo ""
        echo "[meta]"
        echo "artifact = \"ltee-guidestone\""
        echo "generated = \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\""
        echo "arch = \"$ARCH\""
        echo ""
    } > "$MANIFEST"

    cd "$TARGET"
    if [ -d "artifact/data" ]; then
        find artifact/data -type f | sort | while read -r f; do
            hash=$(b3sum --no-names "$f")
            echo "[[file]]"
            echo "path = \"$f\""
            echo "blake3 = \"$hash\""
            echo ""
        done >> "$MANIFEST"
    fi
    if [ -d "figures" ]; then
        find figures -type f -name "*.svg" | sort | while read -r f; do
            hash=$(b3sum --no-names "$f")
            echo "[[file]]"
            echo "path = \"$f\""
            echo "blake3 = \"$hash\""
            echo ""
        done >> "$MANIFEST"
    fi
    cd "$ROOT"

    HASH_COUNT=$(grep -c '^\[\[file\]\]' "$MANIFEST" 2>/dev/null || echo 0)
    log "data_manifest.toml generated ($HASH_COUNT files hashed)"
else
    log "WARNING: b3sum not found — data_manifest.toml not generated"
    log "  Install with: cargo install b3sum"
    touch "$TARGET/data_manifest.toml"
fi

# --- Summary ---
step "Assembly Complete"
USB_SIZE=$(du -sh "$TARGET" | cut -f1)
BIN_COUNT=$(find "$TARGET/bin" -type f 2>/dev/null | wc -l)
DATA_COUNT=$(find "$TARGET/artifact/data" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | wc -l)
NB_COUNT=$(find "$TARGET/notebooks" -name "*.py" 2>/dev/null | wc -l)
FIG_TOTAL=$(find "$TARGET/figures" -name "*.svg" 2>/dev/null | wc -l)
HAS_PYTHON=$( [ -f "$TARGET/python/bin/python3" ] && echo "yes" || echo "no" )

echo ""
echo "  Target:     $TARGET"
echo "  Size:       $USB_SIZE"
echo "  Binaries:   $BIN_COUNT ecoBin modules"
echo "  Data:       $DATA_COUNT datasets"
echo "  Notebooks:  $NB_COUNT Python baselines"
echo "  Figures:    $FIG_TOTAL SVG scientific figures"
echo "  Python:     $HAS_PYTHON (embedded)"
echo "  Provenance: liveSpore.json ($(cat "$TARGET/liveSpore.json" | python3 -c 'import sys,json; print(len(json.load(sys.stdin)))' 2>/dev/null || echo 0) entries)"
echo "  Marker:     .biomeos-spore ($(cat "$TARGET/.biomeos-spore" | python3 -c 'import sys,json; d=json.load(sys.stdin); print(d["class"])' 2>/dev/null || echo present))"
echo ""
echo "  To validate: cd $TARGET && ./validate"
echo "  To refresh:  cd $TARGET && ./refresh"
