# ── Builder ──────────────────────────────────────────────────────────────
FROM rust:1-slim AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
RUN cargo build --release --bin kagoroute-engine

# ── Runtime ──────────────────────────────────────────────────────────────
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/kagoroute-engine /usr/local/bin/kagoroute-engine

EXPOSE 8080
CMD ["kagoroute-engine"]
