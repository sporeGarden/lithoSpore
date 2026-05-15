#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# build-container.sh — Build lithoSpore OCI container image
#
# Usage:
#   ./scripts/build-container.sh                    # Build with podman
#   ./scripts/build-container.sh --engine docker     # Build with docker
#   ./scripts/build-container.sh --tag my-tag        # Custom tag
#   ./scripts/build-container.sh --push              # Push to registry after build
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(dirname "$SCRIPT_DIR")"

ENGINE="podman"
TAG="lithospore:latest"
PUSH=false

while [ $# -gt 0 ]; do
    case "$1" in
        --engine)  ENGINE="$2"; shift 2 ;;
        --tag)     TAG="$2"; shift 2 ;;
        --push)    PUSH=true; shift ;;
        --help|-h)
            echo "Usage: $0 [--engine podman|docker] [--tag TAG] [--push]"
            exit 0
            ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

log() { echo "[build-container] $(date '+%H:%M:%S') $*"; }

if ! command -v "$ENGINE" >/dev/null 2>&1; then
    log "ERROR: $ENGINE not found. Install podman or docker."
    exit 1
fi

log "Building lithoSpore container image"
log "  Engine:  $ENGINE"
log "  Context: $ROOT"
log "  Tag:     $TAG"

cd "$ROOT"

$ENGINE build -t "$TAG" -f Containerfile .

log "Build complete"

IMAGE_SIZE=$($ENGINE image inspect "$TAG" --format '{{.Size}}' 2>/dev/null || echo "unknown")
if [ "$IMAGE_SIZE" != "unknown" ]; then
    IMAGE_MB=$((IMAGE_SIZE / 1024 / 1024))
    log "  Image size: ${IMAGE_MB}MB"
fi

if command -v b3sum >/dev/null 2>&1; then
    IMAGE_ID=$($ENGINE image inspect "$TAG" --format '{{.Id}}' 2>/dev/null | head -c 16)
    log "  Image ID:   ${IMAGE_ID}..."
fi

log ""
log "Run validation:"
log "  $ENGINE run --rm $TAG"
log ""
log "Run validation (JSON):"
log "  $ENGINE run --rm $TAG validate --json"
log ""
log "Interactive shell:"
log "  $ENGINE run --rm -it $TAG /bin/bash"

if $PUSH; then
    log "Pushing $TAG..."
    $ENGINE push "$TAG"
    log "Push complete"
fi
