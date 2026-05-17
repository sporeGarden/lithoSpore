#!/bin/sh
# Bundled Python wrapper — routes to the standalone interpreter
# shipped inside this artifact. No host Python required.
#
# Usage: ./python <script.py> [args...]
#
# The standalone Python includes numpy, scipy, and matplotlib.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PYTHON_BIN="$SCRIPT_DIR/python/bin/python3.13"

if [ ! -x "$PYTHON_BIN" ]; then
    echo "error: bundled Python not found at $PYTHON_BIN" >&2
    echo "       The python/ directory should be alongside this script." >&2
    exit 1
fi

exec "$PYTHON_BIN" "$@"
