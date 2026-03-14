# ── Build stage ───────────────────────────────────────────────
FROM rust:1.89-bookworm AS builder

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY crates/tmux-gateway-core/Cargo.toml crates/tmux-gateway-core/Cargo.toml
RUN mkdir src && echo "fn main() {}" > src/main.rs && mkdir -p crates/tmux-gateway-core/src && echo "" > crates/tmux-gateway-core/src/lib.rs && cargo build --release && rm -rf src crates/tmux-gateway-core/src

# Build the real project
COPY schemas ./schemas
COPY crates ./crates
COPY src ./src
RUN find . -name '*.rs' -exec touch {} + && cargo build --release

# ── Runtime stage ─────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y tmux curl && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/tmux-gateway /usr/local/bin/tmux-gateway

RUN useradd -r -s /bin/false tmux-gateway
USER tmux-gateway

ENV HTTP_PORT=8080
ENV GRPC_PORT=50051

EXPOSE 8080 50051

HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
  CMD curl -f http://localhost:${HTTP_PORT}/health || exit 1

CMD ["tmux-gateway"]
