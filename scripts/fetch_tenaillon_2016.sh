#!/usr/bin/env bash
# fetch_tenaillon_2016.sh — Fetch Tenaillon et al. 2016 264-genome breseq data
# BioProject: PRJNA294072 (public-domain)
# Paper: Tenaillon O et al. (2016) Tempo and mode of genome evolution in a
#        50,000-generation experiment. Nature 536:165-170.
# wetSpring source: experiments/results/ltee_b7_expected_values.json
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ARTIFACT_ROOT="${SCRIPT_DIR}/../artifact"
DATA_DIR="${ARTIFACT_ROOT}/data/tenaillon_2016"
WETSPRING_SRC="${ECOPRIMALS_ROOT:-$GATE_HOME/Development/ecoPrimals}/springs/wetSpring"

log() { echo "[fetch_tenaillon_2016] $(date '+%H:%M:%S') $*"; }

mkdir -p "$DATA_DIR"

log "Fetching Tenaillon 2016 LTEE 264-genome breseq data"
log "  BioProject: PRJNA294072"
log "  wetSpring source: ${WETSPRING_SRC}"
log "  Target: ${DATA_DIR}"

EXPECTED_JSON="${WETSPRING_SRC}/experiments/results/ltee_b7_expected_values.json"
if [[ -f "$EXPECTED_JSON" ]]; then
    log "Copying expected values from wetSpring B7..."
    cp "$EXPECTED_JSON" "$DATA_DIR/expected_values.json"
    log "  Copied ltee_b7_expected_values.json → expected_values.json"
else
    log "WARN: wetSpring B7 expected values not found at $EXPECTED_JSON"
    log "  Set ECOPRIMALS_ROOT or GATE_HOME, or fetch manually"
fi

MUTATION_CURVES="${WETSPRING_SRC}/experiments/results/ltee_b7_mutation_curves.json"
if [[ -f "$MUTATION_CURVES" ]]; then
    cp "$MUTATION_CURVES" "$DATA_DIR/mutation_curves.json"
    log "  Copied mutation_curves.json"
fi

log "Creating SRA download instructions for full 264-genome dataset..."
cat > "${DATA_DIR}/README_fetch.md" << 'HEREDOC'
# Tenaillon 2016 — Full 264-Genome Dataset

BioProject PRJNA294072 contains whole-genome sequencing data for 264 clones
from 12 LTEE populations sampled at multiple timepoints.

## Quick fetch (SRA Toolkit required)

```bash
# Fetch all 264 genomes (~50GB raw)
prefetch --option-file accession_list.txt

# Convert to FASTQ
for sra in *.sra; do
    fastq-dump --split-files --gzip "$sra"
done
```

## breseq analysis

```bash
# Reference: REL606 (NC_012967)
# Run breseq per-clone:
for r1 in *_1.fastq.gz; do
    sample=$(basename "$r1" _1.fastq.gz)
    breseq -r REL606.gbk "${sample}_1.fastq.gz" "${sample}_2.fastq.gz" -o "breseq_${sample}"
done
```

For Tier 1 validation, the expected_values.json from wetSpring is sufficient.
Full SRA data enables Tier 2 breseq re-analysis.
HEREDOC

if ! command -v b3sum >/dev/null 2>&1; then
    log "WARN: b3sum not found — skipping BLAKE3 hash."
    log "  Install: cargo install b3sum"
else
    if [[ -f "$DATA_DIR/expected_values.json" ]]; then
        log "Computing BLAKE3 hashes..."
        HASH=$(find "$DATA_DIR" -type f | sort | xargs cat | b3sum | cut -d' ' -f1)
        TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
        log "  Dataset BLAKE3: ${HASH}"
        log "  Retrieved: ${TIMESTAMP}"
    fi
fi

log "Done — module 6 data staged for lithoSpore ingestion."
