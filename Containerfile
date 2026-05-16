# SPDX-License-Identifier: AGPL-3.0-or-later
#
# lithoSpore OCI Container — Containerized LTEE Targeted GuideStone
#
# Multi-stage build:
#   Stage 1: Compile ecoBin modules as musl-static binaries
#   Stage 2: Minimal runtime with Python + data + binaries
#
# Usage:
#   podman build -t lithospore:latest .
#   podman run --rm lithospore:latest
#   podman run --rm lithospore:latest litho validate --json

# ── Stage 1: Rust builder ───────────────────────────────────────────

FROM docker.io/library/rust:1.85-bookworm AS builder

RUN apt-get update && apt-get install -y musl-tools && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/

RUN cargo build --release --target x86_64-unknown-linux-musl \
    && mkdir -p /out/bin \
    && cp "target/x86_64-unknown-linux-musl/release/litho" /out/bin/ \
    && strip /out/bin/litho

# ── Stage 2: Runtime ────────────────────────────────────────────────

FROM docker.io/library/python:3.12-slim-bookworm

LABEL org.opencontainers.image.title="lithoSpore"
LABEL org.opencontainers.image.description="LTEE Targeted GuideStone — containerized validation"
LABEL org.opencontainers.image.source="https://github.com/sporeGarden/lithoSpore"
LABEL org.opencontainers.image.licenses="AGPL-3.0-or-later"

RUN pip install --no-cache-dir numpy scipy

WORKDIR /lithoSpore

COPY --from=builder /out/bin/ bin/

COPY artifact/data/ artifact/data/
COPY artifact/data.toml artifact/
COPY artifact/scope.toml artifact/
COPY artifact/tolerances.toml artifact/

COPY validation/expected/ validation/expected/

COPY notebooks/ notebooks/

COPY artifact/usb-root/.biomeos-spore .biomeos-spore
RUN ln -s bin/litho validate && \
    ln -s bin/litho verify && \
    ln -s bin/litho refresh && \
    ln -s bin/litho spore

RUN echo '[]' > liveSpore.json

ENV PATH="/lithoSpore/bin:${PATH}"

ENTRYPOINT ["bin/litho"]
CMD ["validate"]
