#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# fetch_dfe_2024.sh — Fetch DFE evolution data for Anderson-QS predictions
# Paper: DFE Evolution in LTEE 2024, Science
# Status: UPSTREAM-BLOCKED — source URI pending publication
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ARTIFACT_ROOT="${SCRIPT_DIR}/../artifact"
DATA_DIR="${ARTIFACT_ROOT}/data/dfe_2024"

log() { echo "[fetch_dfe_2024] $(date '+%H:%M:%S') $*"; }

mkdir -p "$DATA_DIR"

log "DFE evolution dataset for Anderson-QS module"
log "  Status: UPSTREAM-BLOCKED"
log "  The paper's DOI and data repository URI are not yet available."
log "  Once published, this script will fetch DFE shape parameter"
log "  timeseries and fitness landscape measurements."
log ""
log "  Upstream dependencies:"
log "    - hotSpring B2: Anderson disorder analogy predictions"
log "    - groundSpring: baseline DFE estimation"

cat > "${DATA_DIR}/README_fetch.md" << 'HEREDOC'
# DFE Evolution Data — Anderson-QS Module

## Status: Upstream-Blocked

The DFE evolution dataset is not yet publicly available.
Module 7 (ltee-anderson) currently validates against internally
generated predictions from hotSpring + groundSpring.

## Expected data format

- `dfe_timeseries.csv`: gamma shape parameter across LTEE timepoints
- `fitness_landscape.json`: local fitness measurements per population
- `expected_values.json`: validation targets from spring reproductions

## Interim

Module 7 uses `anderson_predictions/` (generated data) for validation.
This script will supplement with external DFE data once published.
HEREDOC

log "Stub data directory created at ${DATA_DIR}"
log "Module 7 uses anderson_predictions/ for validation in the interim."
