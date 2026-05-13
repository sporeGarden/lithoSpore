#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# fetch_blount_2012.sh — Fetch Blount et al. 2008/2012 citrate innovation data
# BioProject: PRJNA188627 (public-domain)
# Papers: Blount ZD et al. (2008) Historical contingency and the evolution of a
#         key innovation in an experimental population of E. coli. PNAS 105:7899-7906.
#         Blount ZD et al. (2012) Genomic analysis of a key innovation in an
#         experimental E. coli population. Nature 489:513-518.
# groundSpring control: control/ltee_citrate_innovation/
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ARTIFACT_ROOT="${SCRIPT_DIR}/../artifact"
DATA_DIR="${ARTIFACT_ROOT}/data/blount_2012"
EXPECTED_SRC="${ECOPRIMALS_ROOT:-$GATE_HOME/Development/ecoPrimals}/springs/groundSpring/control/ltee_citrate_innovation"

log() { echo "[fetch_blount_2012] $(date '+%H:%M:%S') $*"; }

mkdir -p "$DATA_DIR"

log "Fetching Blount 2008/2012 LTEE citrate innovation data"
log "  BioProject: PRJNA188627"
log "  groundSpring source: ${EXPECTED_SRC}"
log "  Target: ${DATA_DIR}"

if [[ -d "$EXPECTED_SRC" ]]; then
    log "Copying expected values from groundSpring B4..."
    cp "$EXPECTED_SRC/expected_values.json" "$DATA_DIR/expected_values.json"
    [[ -f "$EXPECTED_SRC/tolerances.toml" ]] && cp "$EXPECTED_SRC/tolerances.toml" "$DATA_DIR/tolerances.toml"
    log "  Copied expected_values.json + tolerances.toml"

    log "Creating lithoSpore module 4 expected values..."
    python3 -c "
import json
src = json.load(open('${DATA_DIR}/expected_values.json'))
module = {
    'experiment': src['experiment'],
    'paper': src.get('paper', 'Blount2008'),
    'paper_id': src.get('paper_id', 'B4'),
    'litho_module': 4,
    'module_name': 'ltee-citrate',
    'description': 'Citrate innovation — historical contingency and potentiation',
    'cit_plus_fraction': src.get('cit_plus_fraction'),
    'potentiation_fraction': src.get('potentiation_fraction'),
    'mean_potentiation_gen': src.get('mean_potentiation_gen'),
    'mean_cit_plus_gen': src.get('mean_cit_plus_gen'),
    'replay_probabilities': src.get('replay_probabilities', {}),
    'checks_passed': src.get('checks_passed', 0),
    'checks_total': src.get('checks_total', 0),
    'source': 'groundSpring V140 control/ltee_citrate_innovation',
    'provenance': 'BLAKE3-anchored via lithoSpore fetch_blount_2012.sh'
}
with open('${SCRIPT_DIR}/../validation/expected/module4_citrate.json', 'w') as f:
    json.dump(module, f, indent=2)
print('  Wrote module4_citrate.json')
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

log "Done — module 4 data ready for lithoSpore ingestion."
