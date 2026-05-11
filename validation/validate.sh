#!/bin/sh
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Validation harness entry point.
# Runs each module binary and collects results into a unified report.

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(dirname "$SCRIPT_DIR")"

echo "lithoSpore validation harness"
echo "============================="
echo ""

MODULES="ltee-fitness ltee-mutations ltee-alleles ltee-citrate ltee-biobricks ltee-breseq ltee-anderson"
PASS=0
FAIL=0
SKIP=0

for mod in $MODULES; do
    BIN="$ROOT/artifact/bin/$(uname -m)/static/$mod"
    if [ -f "$BIN" ] && [ -x "$BIN" ]; then
        echo "Running: $mod"
        if "$BIN" --json > "/tmp/litho_${mod}.json" 2>&1; then
            PASS=$((PASS + 1))
            echo "  PASS"
        else
            EXIT=$?
            if [ $EXIT -eq 2 ]; then
                SKIP=$((SKIP + 1))
                echo "  SKIP (Tier 3 unavailable)"
            else
                FAIL=$((FAIL + 1))
                echo "  FAIL"
            fi
        fi
    else
        echo "Skipping: $mod (binary not found at $BIN)"
        SKIP=$((SKIP + 1))
    fi
done

echo ""
echo "Results: $PASS PASS, $FAIL FAIL, $SKIP SKIP"

if [ $FAIL -gt 0 ]; then
    exit 1
elif [ $SKIP -gt 0 ]; then
    exit 2
else
    exit 0
fi
