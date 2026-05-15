#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# fetch_biobricks_2024.sh — Fetch BioBrick metabolic burden data (Burden 2024)
# Paper: Barrick et al. 2024, Nat Comms. doi:10.1038/s41467-024-50639-9
# Source: https://github.com/barricklab/igem2019 (v1.0.2 release)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ARTIFACT_ROOT="${SCRIPT_DIR}/../artifact"
DATA_DIR="${ARTIFACT_ROOT}/data/biobricks_2024"
DATA_TOML="${ARTIFACT_ROOT}/data.toml"

GITHUB_REPO="barricklab/igem2019"
RELEASE_TAG="v1.0.2"
TARBALL_URL="https://github.com/${GITHUB_REPO}/archive/refs/tags/${RELEASE_TAG}.tar.gz"

log() { echo "[fetch_biobricks_2024] $(date '+%H:%M:%S') $*"; }

mkdir -p "$DATA_DIR"

log "BioBrick burden dataset (Barrick et al. 2024, Nat Comms)"
log "  DOI:    10.1038/s41467-024-50639-9"
log "  Source: ${TARBALL_URL}"
log "  Target: ${DATA_DIR}"

if command -v curl >/dev/null 2>&1; then
    FETCHER="curl -fSL --retry 3 --max-time 300 -o"
elif command -v wget >/dev/null 2>&1; then
    FETCHER="wget -q --tries=3 --timeout=300 -O"
else
    echo "ERROR: Neither curl nor wget found." >&2
    exit 1
fi

TARBALL="${DATA_DIR}/.igem2019-${RELEASE_TAG}.tar.gz"
EXTRACT_DIR="${DATA_DIR}/.extract"

if [ -f "${DATA_DIR}/igem2019_part_metadata.csv" ] && \
   [ -f "${DATA_DIR}/igem2019_strain_metadata.csv" ] && \
   [ -f "${DATA_DIR}/igem2019_plate_metadata.csv" ]; then
    log "Data already present — skipping download (delete ${DATA_DIR} to re-fetch)"
else
    log "Downloading ${RELEASE_TAG} tarball..."
    $FETCHER "$TARBALL" "$TARBALL_URL"

    log "Extracting burden_assay data..."
    rm -rf "$EXTRACT_DIR"
    mkdir -p "$EXTRACT_DIR"
    tar xzf "$TARBALL" -C "$EXTRACT_DIR"

    REPO_DIR=$(find "$EXTRACT_DIR" -maxdepth 1 -type d -name "igem2019-*" | head -1)
    if [ -z "$REPO_DIR" ]; then
        log "ERROR: Could not find extracted repo directory"
        exit 1
    fi

    BURDEN_DIR="${REPO_DIR}/burden_assay"
    if [ ! -d "$BURDEN_DIR" ]; then
        log "ERROR: burden_assay directory not found in tarball"
        exit 1
    fi

    cp "${BURDEN_DIR}/igem2019_part_metadata.csv" "${DATA_DIR}/"
    cp "${BURDEN_DIR}/igem2019_strain_metadata.csv" "${DATA_DIR}/"
    cp "${BURDEN_DIR}/igem2019_plate_metadata.csv" "${DATA_DIR}/"
    cp "${BURDEN_DIR}/igem2019_sequencing_results.csv" "${DATA_DIR}/"

    if [ -d "${BURDEN_DIR}/input-plate-data" ]; then
        cp -r "${BURDEN_DIR}/input-plate-data" "${DATA_DIR}/"
    fi

    if [ -d "${BURDEN_DIR}/scripts" ]; then
        cp -r "${BURDEN_DIR}/scripts" "${DATA_DIR}/analysis_scripts/"
    fi

    rm -rf "$EXTRACT_DIR" "$TARBALL"
    log "Extraction complete"
fi

log "Verifying data integrity..."
PART_COUNT=$(tail -n +2 "${DATA_DIR}/igem2019_part_metadata.csv" | wc -l)
log "  Parts in metadata: ${PART_COUNT}"
if [ "$PART_COUNT" -lt 300 ]; then
    log "ERROR: Expected ~301 parts, got ${PART_COUNT}"
    exit 1
fi

if ! command -v b3sum >/dev/null 2>&1; then
    log "WARN: b3sum not found — skipping BLAKE3 hash (install: cargo install b3sum)"
    log "Done (without hash)."
    exit 0
fi

log "Computing BLAKE3 hashes..."
HASH=$(find "$DATA_DIR" -type f -not -name '.*' | sort | xargs cat | b3sum | cut -d' ' -f1)
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
log "  Dataset BLAKE3: ${HASH}"
log "  Retrieved: ${TIMESTAMP}"
log ""
log "  Update artifact/data.toml:"
log "    blake3 = \"${HASH}\""
log "    retrieved = \"${TIMESTAMP}\""

log "Done."
