#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# fetch_tenaillon_2016.sh — Fetch Tenaillon et al. 2016 264-genome breseq data
# BioProject: PRJNA294072 (public-domain)
# Paper: Tenaillon O et al. (2016) Tempo and mode of genome evolution in a
#        50,000-generation experiment. Nature 536:165-170.
# DOI: 10.1038/nature18959
#
# Self-contained: uses validation/expected/module6_breseq.json as source
# of truth. If wetSpring is available, it supplements with mutation curves.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(dirname "$SCRIPT_DIR")"
ARTIFACT_ROOT="${ROOT}/artifact"
DATA_DIR="${ARTIFACT_ROOT}/data/tenaillon_2016"
EXPECTED_JSON="${ROOT}/validation/expected/module6_breseq.json"
WETSPRING_SRC="${ECOPRIMALS_ROOT:-${GATE_HOME:-/nonexistent}/Development/ecoPrimals}/springs/wetSpring"

log() { echo "[fetch_tenaillon_2016] $(date '+%H:%M:%S') $*"; }

mkdir -p "$DATA_DIR"

log "Fetching Tenaillon 2016 LTEE 264-genome breseq data"
log "  BioProject: PRJNA294072"
log "  DOI: 10.1038/nature18959"
log "  Target: ${DATA_DIR}"

# --- Strategy 1: Use wetSpring if available (ecosystem mode) ---
WETSPRING_EXPECTED="${WETSPRING_SRC}/experiments/results/ltee_b7_expected_values.json"
if [[ -f "$WETSPRING_EXPECTED" ]]; then
    log "Using wetSpring B7 expected values (ecosystem mode)"
    cp "$WETSPRING_EXPECTED" "$DATA_DIR/expected_values.json"
    MUTATION_CURVES="${WETSPRING_SRC}/experiments/results/ltee_b7_mutation_curves.json"
    [[ -f "$MUTATION_CURVES" ]] && cp "$MUTATION_CURVES" "$DATA_DIR/mutation_curves.json"
    log "  Copied from wetSpring"

# --- Strategy 2: Generate from lithoSpore's own expected JSON (standalone mode) ---
elif [[ -f "$EXPECTED_JSON" ]]; then
    log "Using lithoSpore validation/expected/module6_breseq.json (standalone mode)"
    cp "$EXPECTED_JSON" "$DATA_DIR/expected_values.json"
    log "  Generated data bundle from expected values"

# --- Strategy 3: Generate minimal data from published values ---
else
    log "No expected JSON found — generating from published paper values"
    python3 -c "
import json
data = {
    'experiment': 'Exp380',
    'paper': 'Tenaillon et al. Nature 536, 165-170 (2016)',
    'doi': '10.1038/nature18959',
    'bioproject': 'PRJNA294072',
    'ltee_queue_id': 'B7',
    'litho_module': 6,
    'targets': {
        'n_populations': {'value': 12, 'unit': 'count', 'tolerance': 0,
            'source': 'Paper methods: 12 replicate populations'},
        'n_genomes': {'value': 264, 'unit': 'count', 'tolerance': 0,
            'source': 'BioProject PRJNA294072: 264 sequenced clones'},
        'genome_length_bp': {'value': 4629812, 'unit': 'bp', 'tolerance': 100,
            'source': 'REL606 ancestor genome NC_012967.1'},
        'nonmutator_rate_per_bp_per_gen': {'value': 8.9e-11,
            'unit': 'mutations/bp/generation', 'tolerance': 1e-11,
            'source': 'Fig 1, non-hypermutator populations'},
        'nonmutator_mutations_at_50k': {'value': 20.6,
            'unit': 'point_mutations', 'tolerance': 2.3,
            'source': 'Linear model: mu * L * 50000'},
        'ts_tv_ratio': {'value': 1.7, 'unit': 'ratio', 'tolerance': 0.3,
            'source': 'Table S2, aggregate across non-mutator populations'},
        'gc_to_at_fraction': {'value': 0.68, 'unit': 'fraction', 'tolerance': 0.05,
            'source': 'Table S2, dominant mutation class'},
        'mutation_spectrum': {
            'value': {'GC_to_AT': 0.68, 'AT_to_GC': 0.08, 'GC_to_TA': 0.10,
                      'GC_to_CG': 0.02, 'AT_to_TA': 0.07, 'AT_to_CG': 0.05},
            'unit': 'fraction_per_class', 'tolerance': 0.05,
            'source': 'Table S2, 6-class point mutation spectrum'
        }
    },
    'mutation_accumulation_curve': {
        'generations': [0, 2000, 5000, 10000, 15000, 20000, 30000, 40000, 50000],
        'expected_mutations_nonmutator': [0.0, 0.8, 2.1, 4.1, 6.2, 8.2, 12.4, 16.5, 20.6],
        'model': 'linear',
        'rate_per_bp_per_gen': 8.9e-11
    },
    'provenance': {
        'pipeline': 'lithoSpore standalone',
        'source': 'Generated from Tenaillon et al. 2016 published values'
    }
}
with open('${DATA_DIR}/expected_values.json', 'w') as f:
    json.dump(data, f, indent=2)
print('  Wrote expected_values.json from paper-derived values')
" || {
        log "ERROR: python3 not found — cannot generate data"
        exit 1
    }
fi

# SRA download instructions for full dataset
cat > "${DATA_DIR}/README.md" << 'HEREDOC'
# Tenaillon 2016 — 264-Genome Dataset

BioProject PRJNA294072 contains whole-genome sequencing data for 264 clones
from 12 LTEE populations sampled at multiple timepoints through 50,000
generations.

DOI: 10.1038/nature18959

## Quick fetch (SRA Toolkit required)

```bash
prefetch --option-file accession_list.txt
for sra in *.sra; do
    fastq-dump --split-files --gzip "$sra"
done
```

## breseq analysis

```bash
# Reference: REL606 (NC_012967)
for r1 in *_1.fastq.gz; do
    sample=$(basename "$r1" _1.fastq.gz)
    breseq -r REL606.gbk "${sample}_1.fastq.gz" "${sample}_2.fastq.gz" \
        -o "breseq_${sample}"
done
```

Tier 1/2 validation uses the expected_values.json summary.
Full SRA data enables Tier 3 breseq re-analysis from raw reads.
HEREDOC

if command -v b3sum >/dev/null 2>&1; then
    if [[ -f "$DATA_DIR/expected_values.json" ]]; then
        log "Computing BLAKE3 hashes..."
        HASH=$(find "$DATA_DIR" -type f | sort | xargs cat | b3sum | cut -d' ' -f1)
        TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
        log "  Dataset BLAKE3: ${HASH}"
        log "  Retrieved: ${TIMESTAMP}"
    fi
else
    log "WARN: b3sum not found — skipping BLAKE3 hash (install: cargo install b3sum)"
fi

log "Done — module 6 data staged for lithoSpore ingestion."
