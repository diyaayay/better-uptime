# syntax=docker/dockerfile:1

# --- build ---
FROM rust:bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY api ./api
COPY store ./store

RUN cargo build --release -p api

# --- runtime ---
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libpq5 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/api /usr/local/bin/api

ENV RUST_BACKTRACE=1
EXPOSE 3000

USER nobody
CMD ["/usr/local/bin/api"]
