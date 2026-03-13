# ── Build stage ───────────────────────────────────────────────
FROM rust:1.89-bookworm AS builder

RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY crates/tmux-gateway-core/Cargo.toml crates/tmux-gateway-core/Cargo.toml
RUN mkdir src && echo "fn main() {}" > src/main.rs && mkdir -p crates/tmux-gateway-core/src && echo "" > crates/tmux-gateway-core/src/lib.rs && cargo build --release && rm -rf src crates/tmux-gateway-core/src

# Build the real project
COPY schemas ./schemas
COPY crates ./crates
COPY src ./src
RUN touch src/main.rs && cargo build --release

# ── Runtime stage ─────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y tmux && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/tmux-gateway /usr/local/bin/tmux-gateway

ENV HTTP_PORT=3020
ENV GRPC_PORT=50251

EXPOSE 3020 50251

CMD ["tmux-gateway"]
