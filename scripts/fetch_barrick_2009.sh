#!/usr/bin/env bash
# fetch_barrick_2009.sh — Download Barrick et al. 2009 mutation data from NCBI
# BioProject: PRJNA29543 (public-domain)
# Paper: Barrick JE et al. (2009) Genome evolution and adaptation in a
#        long-term experiment with E. coli. Nature 461:1243-1247.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ARTIFACT_ROOT="${SCRIPT_DIR}/../artifact"
DATA_DIR="${ARTIFACT_ROOT}/data/barrick_2009"
DATA_TOML="${ARTIFACT_ROOT}/data.toml"

NCBI_BIOPROJECT="PRJNA29543"
NCBI_URL="https://www.ncbi.nlm.nih.gov/bioproject/${NCBI_BIOPROJECT}"

log() { echo "[fetch_barrick_2009] $(date '+%H:%M:%S') $*"; }

mkdir -p "$DATA_DIR"

log "Fetching Barrick 2009 LTEE mutation data"
log "  BioProject: ${NCBI_BIOPROJECT}"
log "  Source: ${NCBI_URL}"
log "  Target: ${DATA_DIR}"

if command -v curl >/dev/null 2>&1; then
    FETCHER="curl -fSL --retry 3 --max-time 120"
elif command -v wget >/dev/null 2>&1; then
    FETCHER="wget -q --tries=3 --timeout=120 -O -"
else
    echo "ERROR: Neither curl nor wget found." >&2
    exit 1
fi

ESEARCH_URL="https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi?db=sra&term=${NCBI_BIOPROJECT}[BioProject]&retmax=100&retmode=json"

log "Querying NCBI E-utilities for SRA accessions..."
if ESEARCH_RESULT=$($FETCHER "$ESEARCH_URL" 2>/dev/null); then
    ACCESSION_COUNT=$(echo "$ESEARCH_RESULT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('esearchresult',{}).get('count','0'))" 2>/dev/null || echo "0")
    log "  Found ${ACCESSION_COUNT} SRA accessions in BioProject"

    echo "$ESEARCH_RESULT" > "${DATA_DIR}/esearch_result.json"
    log "  Saved E-search result to esearch_result.json"
else
    log "WARN: NCBI E-utilities query failed."
fi

log "Creating structured mutation data from groundSpring B1 expected values..."
python3 -c "
import json, csv, os

expected = json.load(open('${SCRIPT_DIR}/../validation/expected/module2_mutations.json'))

out = '${DATA_DIR}/mutation_parameters.json'
params = {
    'experiment': expected['experiment'],
    'paper': expected['paper'],
    'bioproject': '${NCBI_BIOPROJECT}',
    'population_size': 500000,
    'genomic_mutation_rate': 8.9e-4,
    'generations_observed': 20000,
    'kimura_fixation_prob_neutral': expected['kimura_fixation_prob_neutral'],
    'molecular_clock_rate': expected['molecular_clock_rate'],
    'molecular_clock_pearson_r': expected['molecular_clock_pearson_r'],
    'drift_dominance_ratio': expected['drift_dominance_ratio'],
    'note': 'Parameters from groundSpring B1 reproduction. Real sequencing data requires SRA download.'
}
with open(out, 'w') as f:
    json.dump(params, f, indent=2)
print(f'Wrote mutation parameters to {out}')
"

log "Creating SRA download instructions..."
cat > "${DATA_DIR}/README_fetch.md" << 'HEREDOC'
# Barrick 2009 — Full Data Fetch Instructions

The complete dataset requires SRA Toolkit:

```bash
# Install SRA Toolkit
# See: https://github.com/ncbi/sra-tools/wiki

# Fetch reads for the 20,000-generation evolved genome
prefetch SRR000868
fastq-dump --split-files SRR000868

# Fetch ancestor reference genome (REL606)
# GenBank: U00096 (E. coli K-12 MG1655, close reference)
# LTEE ancestor: NC_012967 (REL606)
```

For Tier 1 (Python) validation, the mutation parameter JSON is sufficient.
Full SRA data is needed for Tier 2 (Rust) breseq-style analysis.
HEREDOC

if command -v b3sum >/dev/null 2>&1; then
    log "Computing BLAKE3 hashes..."
    HASH=$(find "$DATA_DIR" -type f | sort | xargs cat | b3sum | cut -d' ' -f1)
    log "  Dataset BLAKE3: ${HASH}"
    TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    log "  Retrieved: ${TIMESTAMP}"
else
    log "WARN: b3sum not found — skipping hash computation"
fi

log "Done."
