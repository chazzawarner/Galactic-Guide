# ---------------------------------------------------------------------------
# apps/worker/Dockerfile
# Multi-stage build for the Rust + Nyx propagation worker.
# Targets: base, production, dev
# ---------------------------------------------------------------------------

# ── base ─────────────────────────────────────────────────────────────────────
FROM rust:slim AS base

WORKDIR /app

# System build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests first for dependency layer caching
COPY Cargo.toml rust-toolchain.toml ./
COPY apps/worker/Cargo.toml apps/worker/
COPY crates/viewer/Cargo.toml crates/viewer/

# Create dummy source files so cargo can fetch & cache dependencies
RUN mkdir -p apps/worker/src crates/viewer/src/bin \
    && echo 'fn main() {}' > apps/worker/src/main.rs \
    && echo 'fn main() {}' > crates/viewer/src/main.rs \
    && echo 'fn main() {}' > crates/viewer/src/bin/lunar_transfers.rs

# Fetch dependencies (cached layer)
RUN SQLX_OFFLINE=true cargo fetch

# ── production ───────────────────────────────────────────────────────────────
FROM base AS production-builder

# Copy sqlx offline data before compiling
COPY apps/worker/sqlx-data.json apps/worker/

# Copy real sources
COPY apps/worker/src apps/worker/src
COPY crates/viewer/src crates/viewer/src

RUN SQLX_OFFLINE=true cargo build --release --package worker

# Slim final image
FROM debian:bookworm-slim AS production

WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends libssl3 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=production-builder /app/target/release/worker ./worker

CMD ["./worker"]

# ── dev ──────────────────────────────────────────────────────────────────────
FROM base AS dev

RUN cargo install cargo-watch

COPY apps/worker/sqlx-data.json apps/worker/
COPY apps/worker/src apps/worker/src
COPY crates/viewer/src crates/viewer/src

CMD ["cargo", "watch", "-x", "run --package worker"]
