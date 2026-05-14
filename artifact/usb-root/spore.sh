#!/bin/sh
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# spore.sh — biomeOS ColdSpore/LiveSpore entry point.
#
# biomeOS detects .biomeos-spore and calls this script to orchestrate
# the spore. If biomeOS is not present, falls back to ./validate.

set -e

SPORE_ROOT="$(cd "$(dirname "$0")" && pwd)"

if [ -n "$BIOMEOS_ORCHESTRATOR" ]; then
    echo "lithoSpore: biomeOS orchestration detected"
    echo "  Spore class: hypogeal-cotyledon"
    echo "  Graph: biomeOS/graphs/lithoSpore_validation.toml"

    if [ -f "$SPORE_ROOT/biomeOS/graphs/lithoSpore_validation.toml" ]; then
        echo "  Delegating to biomeOS deploy graph..."
        exit 0
    fi
fi

echo "lithoSpore: standalone mode (no biomeOS)"
exec "$SPORE_ROOT/validate" "$@"
