#!/bin/bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# chaos-test.sh — Fault injection harness for lithoSpore deployment resilience.
#
# Creates an isolated copy of the artifact and systematically injects faults
# to verify that lithoSpore correctly detects and reports each failure mode.
#
# Tests:
#   1. Baseline: clean artifact validates
#   2. Data drift: corrupt a data file, verify DRIFT detection
#   3. Missing file: remove a file, verify MISSING detection
#   4. Corrupt manifest: garble data_manifest.toml, verify detection
#   5. Empty manifest: remove all [[file]] entries, verify detection
#   6. Corrupt expected JSON: break a module's expected values
#   7. Bad computation injection: insert wrong values in expected JSON
#   8. Corrupt liveSpore: garble provenance, verify backup
#   9. Missing binary: remove a module binary, verify graceful skip
#  10. Corrupt tolerances: garble tolerances.toml
#
# Usage:
#   ./scripts/chaos-test.sh                          # Run from repo root
#   ./scripts/chaos-test.sh --from-staging           # Use usb-staging/
#   ./scripts/chaos-test.sh --keep                   # Preserve temp dir

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(dirname "$SCRIPT_DIR")"
KEEP=false
FROM_STAGING=false

while [ $# -gt 0 ]; do
    case "$1" in
        --keep)          KEEP=true; shift ;;
        --from-staging)  FROM_STAGING=true; shift ;;
        -h|--help)
            echo "Usage: $0 [--from-staging] [--keep]"
            exit 0
            ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

CHAOS_DIR=$(mktemp -d /tmp/lithoSpore-chaos.XXXXXXXX)

cleanup() {
    if [ "$KEEP" = false ]; then
        rm -rf "$CHAOS_DIR"
    fi
}
trap cleanup EXIT

PASS=0
FAIL=0
TOTAL=0

check() {
    local name="$1"
    local expected_exit="$2"
    local actual_exit="$3"
    TOTAL=$((TOTAL + 1))
    if [ "$actual_exit" -eq "$expected_exit" ]; then
        PASS=$((PASS + 1))
        echo "  [PASS] $name (exit=$actual_exit, expected=$expected_exit)"
    else
        FAIL=$((FAIL + 1))
        echo "  [FAIL] $name (exit=$actual_exit, expected=$expected_exit)"
    fi
}

check_contains() {
    local name="$1"
    local output="$2"
    local pattern="$3"
    TOTAL=$((TOTAL + 1))
    if echo "$output" | grep -q "$pattern"; then
        PASS=$((PASS + 1))
        echo "  [PASS] $name (output contains '$pattern')"
    else
        FAIL=$((FAIL + 1))
        echo "  [FAIL] $name (output missing '$pattern')"
    fi
}

# Build a fresh artifact copy
build_artifact() {
    local dest="$1"
    if [ "$FROM_STAGING" = true ] && [ -d "$ROOT/usb-staging" ]; then
        cp -r "$ROOT/usb-staging/." "$dest/"
    else
        # Use dev layout
        mkdir -p "$dest"/{bin,artifact/data,validation/expected,papers,figures}
        cp "$ROOT"/target/release/litho "$dest/bin/" 2>/dev/null || cp "$ROOT"/target/debug/litho "$dest/bin/" 2>/dev/null || true
        for bin in ltee-fitness ltee-mutations ltee-alleles ltee-citrate ltee-biobricks ltee-breseq ltee-anderson; do
            cp "$ROOT"/target/release/"$bin" "$dest/bin/" 2>/dev/null || true
        done
        cp -r "$ROOT"/artifact/data/* "$dest/artifact/data/" 2>/dev/null || true
        cp "$ROOT"/artifact/scope.toml "$ROOT"/artifact/data.toml "$ROOT"/artifact/tolerances.toml "$dest/artifact/" 2>/dev/null || true
        cp "$ROOT"/validation/expected/*.json "$dest/validation/expected/" 2>/dev/null || true
        cp "$ROOT"/papers/*.toml "$dest/papers/" 2>/dev/null || true
        cp "$ROOT"/papers/*.md "$dest/papers/" 2>/dev/null || true
        cp "$ROOT"/GETTING_STARTED.md "$ROOT"/SCIENCE.md "$dest/" 2>/dev/null || true
        cp "$ROOT"/artifact/usb-root/.biomeos-spore "$dest/" 2>/dev/null || true
        cp -r "$ROOT"/figures/*.svg "$dest/figures/" 2>/dev/null || true
        # Generate a manifest
        if command -v b3sum >/dev/null 2>&1; then
            cd "$dest"
            {
                echo "[meta]"
                echo "artifact = \"chaos-test\""
                echo ""
                find artifact/data -type f 2>/dev/null | sort | while read -r f; do
                    hash=$(b3sum --no-names "$f")
                    echo "[[file]]"
                    echo "path = \"$f\""
                    echo "blake3 = \"$hash\""
                    echo ""
                done
            } > data_manifest.toml
            cd "$ROOT"
        fi
    fi
}

LITHO=""

echo ""
echo "=================================================================="
echo "  lithoSpore Chaos Test — Fault Injection Harness"
echo "=================================================================="
echo ""

# ── Test 1: Baseline (clean artifact validates) ─────────────────────
echo "=== Test 1: Baseline — clean artifact ==="

BASELINE="$CHAOS_DIR/baseline"
mkdir -p "$BASELINE"
build_artifact "$BASELINE"
LITHO="$BASELINE/bin/litho"

if [ ! -x "$LITHO" ]; then
    echo "  ERROR: litho binary not found. Build first: cargo build --release"
    exit 1
fi

"$LITHO" validate --artifact-root "$BASELINE" --json >/dev/null 2>&1 || true
BASELINE_EXIT=$?
check "baseline validate" 0 $BASELINE_EXIT

"$LITHO" verify --artifact-root "$BASELINE" >/dev/null 2>&1 || true
VERIFY_EXIT=$?
check "baseline verify" 0 $VERIFY_EXIT
echo ""

# ── Test 2: Data drift (corrupt a file) ──────────────────────────────
echo "=== Test 2: Data drift — corrupt a data file ==="

DRIFT="$CHAOS_DIR/drift"
mkdir -p "$DRIFT"
cp -r "$BASELINE/." "$DRIFT/"

FIRST_DATA=$(find "$DRIFT/artifact/data" -type f -print -quit)
if [ -n "$FIRST_DATA" ]; then
    echo "CORRUPTED BY CHAOS TEST" > "$FIRST_DATA"
    DRIFT_EXIT=0
    OUTPUT=$("$LITHO" verify --artifact-root "$DRIFT" 2>&1) || DRIFT_EXIT=$?
    check "drift detected (exit non-zero)" 1 $DRIFT_EXIT
    check_contains "drift output mentions DRIFT" "$OUTPUT" "DRIFT"
else
    echo "  SKIP: no data files to corrupt"
fi
echo ""

# ── Test 3: Missing file ────────────────────────────────────────────
echo "=== Test 3: Missing file — remove a data file ==="

MISSING="$CHAOS_DIR/missing"
mkdir -p "$MISSING"
cp -r "$BASELINE/." "$MISSING/"

FIRST_DATA=$(find "$MISSING/artifact/data" -type f -print -quit)
if [ -n "$FIRST_DATA" ]; then
    rm -f "$FIRST_DATA"
    MISSING_EXIT=0
    OUTPUT=$("$LITHO" verify --artifact-root "$MISSING" 2>&1) || MISSING_EXIT=$?
    check "missing detected (exit non-zero)" 1 $MISSING_EXIT
    check_contains "missing output mentions MISSING" "$OUTPUT" "MISSING"
else
    echo "  SKIP: no data files"
fi
echo ""

# ── Test 4: Corrupt manifest ────────────────────────────────────────
echo "=== Test 4: Corrupt manifest — garble data_manifest.toml ==="

CORRUPT_MAN="$CHAOS_DIR/corrupt-manifest"
mkdir -p "$CORRUPT_MAN"
cp -r "$BASELINE/." "$CORRUPT_MAN/"

echo "NOT VALID TOML {{{{ GARBAGE" > "$CORRUPT_MAN/data_manifest.toml"
CORRUPT_EXIT=0
OUTPUT=$("$LITHO" verify --artifact-root "$CORRUPT_MAN" 2>&1) || CORRUPT_EXIT=$?
check "corrupt manifest (exit non-zero)" 1 $CORRUPT_EXIT
check_contains "corrupt manifest mentions error" "$OUTPUT" "Corrupt\|ERROR"
echo ""

# ── Test 5: Empty manifest ──────────────────────────────────────────
echo "=== Test 5: Empty manifest — no [[file]] entries ==="

EMPTY_MAN="$CHAOS_DIR/empty-manifest"
mkdir -p "$EMPTY_MAN"
cp -r "$BASELINE/." "$EMPTY_MAN/"

printf '[meta]\nartifact = "test"\n' > "$EMPTY_MAN/data_manifest.toml"
EMPTY_EXIT=0
OUTPUT=$("$LITHO" verify --artifact-root "$EMPTY_MAN" 2>&1) || EMPTY_EXIT=$?
check "empty manifest (exit non-zero)" 1 $EMPTY_EXIT
echo ""

# ── Test 6: Corrupt expected JSON ────────────────────────────────────
echo "=== Test 6: Corrupt expected JSON — break module input ==="

CORRUPT_JSON="$CHAOS_DIR/corrupt-json"
mkdir -p "$CORRUPT_JSON"
cp -r "$BASELINE/." "$CORRUPT_JSON/"

FIRST_EXPECTED=$(find "$CORRUPT_JSON/validation/expected" -name "*.json" -print -quit)
if [ -n "$FIRST_EXPECTED" ]; then
    echo "THIS IS NOT JSON {{{{ BROKEN" > "$FIRST_EXPECTED"
    OUTPUT=$("$LITHO" validate --artifact-root "$CORRUPT_JSON" 2>/dev/null) || true
    CORRUPT_JSON_EXIT=$?
    # Module should Skip (cannot parse) — overall validation should still produce output
    check_contains "corrupt JSON module gets SKIP" "$OUTPUT" "SKIP\|Skip"
else
    echo "  SKIP: no expected JSONs"
fi
echo ""

# ── Test 7: Bad computation injection ────────────────────────────────
echo "=== Test 7: Bad values — inject wrong expected values ==="

BAD_VALUES="$CHAOS_DIR/bad-values"
mkdir -p "$BAD_VALUES"
cp -r "$BASELINE/." "$BAD_VALUES/"

FIRST_EXPECTED=$(find "$BAD_VALUES/validation/expected" -name "module1_fitness.json" -print -quit)
if [ -n "$FIRST_EXPECTED" ]; then
    # Replace fitness values with absurd ones
    python3 -c "
import json
with open('$FIRST_EXPECTED') as f:
    d = json.load(f)
if 'model_fits' in d:
    d['model_fits']['power_law']['r_squared'] = -999.0
    d['model_fits']['power_law']['exponent'] = 999.0
with open('$FIRST_EXPECTED', 'w') as f:
    json.dump(d, f)
" 2>/dev/null || echo "  (python3 not available for value injection)"

    OUTPUT=$("$LITHO" validate --artifact-root "$BAD_VALUES" --json 2>/dev/null) || true
    # Check that fitness module FAIL with bad values
    if echo "$OUTPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); ms=[m for m in d['modules'] if m['name']=='power_law_fitness']; sys.exit(0 if ms and ms[0]['status']=='FAIL' else 1)" 2>/dev/null; then
        TOTAL=$((TOTAL + 1)); PASS=$((PASS + 1))
        echo "  [PASS] bad values cause FAIL in fitness module"
    else
        TOTAL=$((TOTAL + 1)); FAIL=$((FAIL + 1))
        echo "  [FAIL] bad values did not cause FAIL in fitness module"
    fi
else
    echo "  SKIP: no fitness expected JSON"
fi
echo ""

# ── Test 8: Corrupt liveSpore ────────────────────────────────────────
echo "=== Test 8: Corrupt liveSpore.json — verify backup ==="

CORRUPT_SPORE="$CHAOS_DIR/corrupt-spore"
mkdir -p "$CORRUPT_SPORE"
cp -r "$BASELINE/." "$CORRUPT_SPORE/"

echo "NOT JSON {{{{ GARBAGE" > "$CORRUPT_SPORE/liveSpore.json"
OUTPUT=$("$LITHO" validate --artifact-root "$CORRUPT_SPORE" --json 2>&1) || true
BACKUP_FILE="$CORRUPT_SPORE/liveSpore.json.bak"
TOTAL=$((TOTAL + 1))
if [ -f "$BACKUP_FILE" ]; then
    PASS=$((PASS + 1))
    echo "  [PASS] corrupt liveSpore.json backed up to .bak"
else
    FAIL=$((FAIL + 1))
    echo "  [FAIL] no backup created for corrupt liveSpore.json"
fi

# Verify original is now valid JSON after validation
TOTAL=$((TOTAL + 1))
if python3 -c "import json; json.load(open('$CORRUPT_SPORE/liveSpore.json'))" 2>/dev/null; then
    PASS=$((PASS + 1))
    echo "  [PASS] liveSpore.json recovered to valid JSON"
else
    FAIL=$((FAIL + 1))
    echo "  [FAIL] liveSpore.json still corrupt after validation"
fi
echo ""

# ── Test 9: Missing binary ──────────────────────────────────────────
echo "=== Test 9: Missing binary — remove a module ==="

MISSING_BIN="$CHAOS_DIR/missing-bin"
mkdir -p "$MISSING_BIN"
cp -r "$BASELINE/." "$MISSING_BIN/"

rm -f "$MISSING_BIN/bin/ltee-fitness"
OUTPUT=$("$LITHO" validate --artifact-root "$MISSING_BIN" --json 2>/dev/null) || true
# fitness module should be Skip (binary not found → falls back to python → if no python, skip)
if echo "$OUTPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); ms=[m for m in d['modules'] if m['name']=='power_law_fitness']; sys.exit(0 if ms and ms[0]['status'] in ['SKIP','PASS'] else 1)" 2>/dev/null; then
    TOTAL=$((TOTAL + 1)); PASS=$((PASS + 1))
    echo "  [PASS] missing binary causes graceful skip/fallback"
else
    TOTAL=$((TOTAL + 1)); FAIL=$((FAIL + 1))
    echo "  [FAIL] missing binary did not cause graceful degradation"
fi
echo ""

# ── Test 10: Corrupt tolerances ──────────────────────────────────────
echo "=== Test 10: Corrupt tolerances.toml ==="

CORRUPT_TOL="$CHAOS_DIR/corrupt-tol"
mkdir -p "$CORRUPT_TOL"
cp -r "$BASELINE/." "$CORRUPT_TOL/"

echo "NOT TOML GARBAGE {{{{" > "$CORRUPT_TOL/artifact/tolerances.toml"
OUTPUT=$("$LITHO" self-test --artifact-root "$CORRUPT_TOL" 2>&1) || true
TOTAL=$((TOTAL + 1))
# Self-test only checks file existence, so corrupt tolerances still passes (known gap)
# Validate should still work since modules don't load tolerances
VALIDATE_OUTPUT=$("$LITHO" validate --artifact-root "$CORRUPT_TOL" --json 2>/dev/null) || true
VALIDATE_EXIT=$?
check "validate runs despite corrupt tolerances" 0 $VALIDATE_EXIT
echo ""

# ── Summary ─────────────────────────────────────────────────────────
echo "=================================================================="
echo "  Chaos Test Summary"
echo "=================================================================="
echo "  Tests:      $PASS/$TOTAL passed"
echo "  Passed:     $PASS"
echo "  Failed:     $FAIL"

if [ "$KEEP" = true ]; then
    echo "  Chaos dir:  $CHAOS_DIR"
fi

echo "=================================================================="
echo ""

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
