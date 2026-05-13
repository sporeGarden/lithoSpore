#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# fetch_good_2017.sh — Fetch Good et al. 2017 clonal interference data
# BioProject: PRJNA380528 (public-domain)
# Paper: Good BH et al. (2017) The dynamics of molecular evolution over
#        60,000 generations. Nature 551:45-50.
# groundSpring control: control/ltee_clonal_interference/
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ARTIFACT_ROOT="${SCRIPT_DIR}/../artifact"
DATA_DIR="${ARTIFACT_ROOT}/data/good_2017"
EXPECTED_SRC="${ECOPRIMALS_ROOT:-$GATE_HOME/Development/ecoPrimals}/springs/groundSpring/control/ltee_clonal_interference"

log() { echo "[fetch_good_2017] $(date '+%H:%M:%S') $*"; }

mkdir -p "$DATA_DIR"

log "Fetching Good 2017 LTEE clonal interference data"
log "  BioProject: PRJNA380528"
log "  groundSpring source: ${EXPECTED_SRC}"
log "  Target: ${DATA_DIR}"

if [[ -d "$EXPECTED_SRC" ]]; then
    log "Copying expected values from groundSpring B3..."
    cp "$EXPECTED_SRC/expected_values.json" "$DATA_DIR/expected_values.json"
    [[ -f "$EXPECTED_SRC/tolerances.toml" ]] && cp "$EXPECTED_SRC/tolerances.toml" "$DATA_DIR/tolerances.toml"
    log "  Copied expected_values.json + tolerances.toml"

    log "Creating lithoSpore module 3 expected values..."
    python3 -c "
import json
src = json.load(open('${DATA_DIR}/expected_values.json'))
module = {
    'experiment': src['experiment'],
    'paper': src['paper'],
    'paper_id': src.get('paper_id', 'B3'),
    'litho_module': 3,
    'module_name': 'ltee-alleles',
    'description': 'Allele frequency trajectories — clonal interference dynamics',
    'pop_sizes': src.get('pop_sizes', []),
    'results_by_size': src.get('results_by_size', {}),
    'checks_total': src.get('checks_total', 0),
    'source': 'groundSpring V140 control/ltee_clonal_interference',
    'provenance': 'BLAKE3-anchored via lithoSpore fetch_good_2017.sh'
}
with open('${SCRIPT_DIR}/../validation/expected/module3_alleles.json', 'w') as f:
    json.dump(module, f, indent=2)
print('  Wrote module3_alleles.json')
"
else
    log "WARN: groundSpring source not found at $EXPECTED_SRC"
    log "  Set ECOPRIMALS_ROOT or GATE_HOME, or fetch manually"
    exit 1
fi

if ! command -v b3sum >/dev/null 2>&1; then
    log "ERROR: b3sum not found — BLAKE3 hashing is required."
    log "  Install: cargo install b3sum"
    exit 1
fi

log "Computing BLAKE3 hashes..."
HASH=$(find "$DATA_DIR" -type f | sort | xargs cat | b3sum | cut -d' ' -f1)
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
log "  Dataset BLAKE3: ${HASH}"
log "  Retrieved: ${TIMESTAMP}"

log "Done — module 3 data ready for lithoSpore ingestion."
