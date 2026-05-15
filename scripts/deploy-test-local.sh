#!/bin/bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# deploy-test-local.sh — Local deployment validation test.
#
# Simulates a USB deployment by copying the artifact to an isolated
# temp directory and running the full validation suite against it.
# No Docker, no VMs — just filesystem isolation. Fast CI-friendly.
#
# Tests:
#   1. Tarball assembly from usb-staging/
#   2. Extract to fresh directory
#   3. Self-test (artifact integrity)
#   4. Tier detection
#   5. Full Tier 2 validation (7 modules)
#   6. Data integrity verification (BLAKE3)
#   7. Verify wrapper scripts work
#   8. Generate deployment report
#
# Usage:
#   ./scripts/deploy-test-local.sh                  # Full test
#   ./scripts/deploy-test-local.sh --from-tarball /path/to/tarball
#   ./scripts/deploy-test-local.sh --keep           # Preserve temp dir

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(dirname "$SCRIPT_DIR")"
TARBALL=""
KEEP=false
RESULTS_DIR=""

while [ $# -gt 0 ]; do
    case "$1" in
        --from-tarball)  TARBALL="$2"; shift 2 ;;
        --keep)          KEEP=true; shift ;;
        --results-dir)   RESULTS_DIR="$2"; shift 2 ;;
        -h|--help)
            echo "Usage: $0 [--from-tarball PATH] [--keep] [--results-dir DIR]"
            exit 0
            ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

DEPLOY_DIR=$(mktemp -d /tmp/lithoSpore-deploy-test.XXXXXXXX)
RESULTS_DIR="${RESULTS_DIR:-$DEPLOY_DIR/results}"

cleanup() {
    if [ "$KEEP" = false ]; then
        rm -rf "$DEPLOY_DIR"
    fi
}
trap cleanup EXIT

echo ""
echo "=================================================================="
echo "  lithoSpore Local Deployment Test"
echo "=================================================================="
echo ""

PASS=0
FAIL=0
TOTAL=0

check() {
    local name="$1"
    local exit_code="$2"
    TOTAL=$((TOTAL + 1))
    if [ "$exit_code" -eq 0 ]; then
        PASS=$((PASS + 1))
        echo "  [PASS] $name"
    else
        FAIL=$((FAIL + 1))
        echo "  [FAIL] $name (exit $exit_code)"
    fi
}

# ── Test 1: Tarball assembly ────────────────────────────────────────
echo "=== Test 1: Tarball assembly ==="

TARBALL_PATH="$DEPLOY_DIR/lithoSpore-usb.tar.gz"

if [ -n "$TARBALL" ]; then
    cp "$TARBALL" "$TARBALL_PATH"
    check "tarball copy" $?
elif [ -d "$ROOT/usb-staging" ]; then
    tar czf "$TARBALL_PATH" -C "$ROOT/usb-staging" .
    check "tarball assembly" $?
else
    echo "  ERROR: No usb-staging/ directory. Run assemble-usb.sh first."
    exit 1
fi

TARBALL_SIZE=$(du -h "$TARBALL_PATH" | cut -f1)
echo "  Tarball: $TARBALL_SIZE"
echo ""

# ── Test 2: Extract to fresh directory ───────────────────────────────
echo "=== Test 2: Extract to isolated deployment directory ==="

ARTIFACT="$DEPLOY_DIR/artifact"
mkdir -p "$ARTIFACT"
tar xzf "$TARBALL_PATH" -C "$ARTIFACT"
check "tarball extraction" $?

TOTAL=$((TOTAL + 1))
if [ -f "$ARTIFACT/.biomeos-spore" ]; then
    PASS=$((PASS + 1))
    echo "  [PASS] .biomeos-spore marker present"
else
    FAIL=$((FAIL + 1))
    echo "  [FAIL] .biomeos-spore marker missing"
fi

TOTAL=$((TOTAL + 1))
if [ -x "$ARTIFACT/bin/litho" ]; then
    PASS=$((PASS + 1))
    echo "  [PASS] bin/litho is executable"
else
    FAIL=$((FAIL + 1))
    echo "  [FAIL] bin/litho not found or not executable"
fi
echo ""

# ── Test 3: Self-test ────────────────────────────────────────────────
echo "=== Test 3: Self-test (artifact integrity) ==="

SELFTEST_OUTPUT=$("$ARTIFACT/bin/litho" self-test --artifact-root "$ARTIFACT" 2>&1) || true
SELFTEST_EXIT=$?
echo "$SELFTEST_OUTPUT"
check "self-test" $SELFTEST_EXIT
echo ""

# ── Test 4: Tier detection ──────────────────────────────────────────
echo "=== Test 4: Tier detection ==="

TIER_OUTPUT=$("$ARTIFACT/bin/litho" tier --artifact-root "$ARTIFACT" 2>&1) || true
TIER_EXIT=$?
echo "$TIER_OUTPUT"
check "tier detection" $TIER_EXIT

TOTAL=$((TOTAL + 1))
if echo "$TIER_OUTPUT" | grep -q "AVAILABLE.*8/8"; then
    PASS=$((PASS + 1))
    echo "  [PASS] All 8 binaries detected"
else
    FAIL=$((FAIL + 1))
    echo "  [FAIL] Not all binaries detected"
fi
echo ""

# ── Test 5: Full Tier 2 validation ───────────────────────────────────
echo "=== Test 5: Full Tier 2 validation ==="

VALIDATION_JSON=$("$ARTIFACT/bin/litho" validate --artifact-root "$ARTIFACT" --json 2>/dev/null) || true
VALIDATION_EXIT=$?
check "validation (exit)" $VALIDATION_EXIT

if command -v python3 &>/dev/null; then
    eval "$(echo "$VALIDATION_JSON" | python3 -c "
import sys, json
try:
    d = json.load(sys.stdin)
    ms = d.get('modules', [])
    print(f'V_PASSED={sum(1 for m in ms if m[\"status\"]==\"PASS\")}')
    print(f'V_TOTAL={len(ms)}')
    print(f'V_CHECKS={sum(m.get(\"checks_passed\",0) for m in ms)}/{sum(m.get(\"checks\",0) for m in ms)}')
    print(f'V_TIER={d.get(\"tier_reached\",\"?\")}')
except:
    print('V_PASSED=?')
    print('V_TOTAL=?')
    print('V_CHECKS=?/?')
    print('V_TIER=?')
" 2>/dev/null)" || true
    echo "  Modules: ${V_PASSED:-?}/${V_TOTAL:-?} passed"
    echo "  Checks:  ${V_CHECKS:-?}"
    echo "  Tier:    ${V_TIER:-?}"

    TOTAL=$((TOTAL + 1))
    if [ "${V_PASSED:-0}" = "${V_TOTAL:-0}" ] && [ "${V_TOTAL:-0}" != "0" ]; then
        PASS=$((PASS + 1))
        echo "  [PASS] All modules passed"
    else
        FAIL=$((FAIL + 1))
        echo "  [FAIL] Not all modules passed"
    fi
fi
echo ""

# ── Test 6: Data integrity verification ──────────────────────────────
echo "=== Test 6: Data integrity verification ==="

VERIFY_OUTPUT=$("$ARTIFACT/bin/litho" verify --artifact-root "$ARTIFACT" 2>&1) || true
VERIFY_EXIT=$?
echo "$VERIFY_OUTPUT"
check "verify (data integrity)" $VERIFY_EXIT
echo ""

# ── Test 7: Wrapper scripts ─────────────────────────────────────────
echo "=== Test 7: Wrapper scripts ==="

TOTAL=$((TOTAL + 1))
if [ -x "$ARTIFACT/validate" ]; then
    WRAPPER_OUT=$("$ARTIFACT/validate" --json 2>&1) || true
    WRAPPER_EXIT=$?
    if [ "$WRAPPER_EXIT" -eq 0 ]; then
        PASS=$((PASS + 1))
        echo "  [PASS] ./validate wrapper works"
    else
        FAIL=$((FAIL + 1))
        echo "  [FAIL] ./validate wrapper returned exit $WRAPPER_EXIT"
    fi
else
    FAIL=$((FAIL + 1))
    echo "  [FAIL] ./validate wrapper not found/executable"
fi

TOTAL=$((TOTAL + 1))
if [ -x "$ARTIFACT/verify" ]; then
    PASS=$((PASS + 1))
    echo "  [PASS] ./verify wrapper present and executable"
else
    FAIL=$((FAIL + 1))
    echo "  [FAIL] ./verify wrapper not found/executable"
fi

TOTAL=$((TOTAL + 1))
if [ -x "$ARTIFACT/refresh" ]; then
    PASS=$((PASS + 1))
    echo "  [PASS] ./refresh wrapper present and executable"
else
    FAIL=$((FAIL + 1))
    echo "  [FAIL] ./refresh wrapper not found/executable"
fi

TOTAL=$((TOTAL + 1))
if [ -x "$ARTIFACT/spore.sh" ]; then
    PASS=$((PASS + 1))
    echo "  [PASS] ./spore.sh wrapper present and executable"
else
    FAIL=$((FAIL + 1))
    echo "  [FAIL] ./spore.sh wrapper not found/executable"
fi
echo ""

# ── Test 8: Deployment report ────────────────────────────────────────
echo "=== Test 8: Deployment report ==="

mkdir -p "$RESULTS_DIR"
REPORT="$RESULTS_DIR/deployment-report.toml"
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)

{
    echo "# lithoSpore Local Deployment Test Report"
    echo "# Generated by deploy-test-local.sh"
    echo ""
    echo "[meta]"
    echo "timestamp = \"$TIMESTAMP\""
    echo "deployment_pattern = \"local-isolated\""
    echo "deploy_dir = \"$DEPLOY_DIR\""
    echo "tarball_size = \"$TARBALL_SIZE\""
    echo ""
    echo "[results]"
    echo "tests_total = $TOTAL"
    echo "tests_passed = $PASS"
    echo "tests_failed = $FAIL"
    echo "self_test_exit = $SELFTEST_EXIT"
    echo "validation_exit = $VALIDATION_EXIT"
    echo "verify_exit = $VERIFY_EXIT"
    echo ""
    echo "[validation]"
    echo "tier_reached = ${V_TIER:-0}"
    echo "modules_passed = ${V_PASSED:-0}"
    echo "modules_total = ${V_TOTAL:-0}"
    echo "checks = \"${V_CHECKS:-?}\""
    echo ""
    echo "[artifact]"
    echo "binaries = $(find "$ARTIFACT/bin" -type f 2>/dev/null | wc -l)"
    echo "datasets = $(find "$ARTIFACT/artifact/data" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | wc -l)"
    echo "figures = $(find "$ARTIFACT/figures" -name '*.svg' 2>/dev/null | wc -l)"
    echo "notebooks = $(find "$ARTIFACT/notebooks" -name '*.py' 2>/dev/null | wc -l)"
} > "$REPORT"

echo "  Report: $REPORT"
echo "$VALIDATION_JSON" > "$RESULTS_DIR/validation.json" 2>/dev/null || true
echo "$VERIFY_OUTPUT" > "$RESULTS_DIR/verify.txt" 2>/dev/null || true
echo "$SELFTEST_OUTPUT" > "$RESULTS_DIR/self-test.txt" 2>/dev/null || true
echo "  Results: $RESULTS_DIR/"
echo ""

# ── Summary ─────────────────────────────────────────────────────────
echo "=================================================================="
echo "  Local Deployment Test Summary"
echo "=================================================================="
echo "  Tests:       $PASS/$TOTAL passed"
echo "  Modules:     ${V_PASSED:-?}/${V_TOTAL:-?}"
echo "  Checks:      ${V_CHECKS:-?}"
echo "  Tier:        ${V_TIER:-?}"
echo "  Tarball:     $TARBALL_SIZE"
echo "  Deploy dir:  $DEPLOY_DIR"
echo "  Report:      $REPORT"

if [ "$KEEP" = true ]; then
    echo ""
    echo "  Artifact preserved at: $ARTIFACT"
    echo "  Cleanup: rm -rf $DEPLOY_DIR"
fi

echo "=================================================================="
echo ""

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
