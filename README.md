# tmux-gateway

A Rust server that exposes a unified interface for interacting with your local [tmux](https://github.com/tmux/tmux) sessions through **gRPC**, **GraphQL**, and **REST** — all from a single process.

## Why?

Managing tmux programmatically typically means shelling out to `tmux` commands and parsing text output. tmux-gateway wraps that complexity behind three well-defined API layers so any client — CLI tool, web dashboard, IDE plugin, or automation script — can interact with tmux using the protocol that fits best.

## Architecture

```
┌───────────────────────────────────────────┐
│               tmux-gateway                │
│                                           │
│  :8080  ┌─────────┐  ┌──────────────┐    │
│ ────────▸  REST    │  │   GraphQL    │    │
│         └────┬─────┘  └──────┬───────┘    │
│              │               │            │
│  :50051 ┌───┴───────────────┴──────┐      │
│ ────────▸           gRPC           │      │
│         └──────────┬───────────────┘      │
│                    │                      │
│           ┌────────▾────────┐             │
│           │   tmux (local)  │             │
│           └─────────────────┘             │
└───────────────────────────────────────────┘
```

| Protocol | Port  | Use case |
|----------|-------|----------|
| REST     | 8080  | Simple integrations, curl, scripts |
| GraphQL  | 8080  | Flexible queries, web UIs (includes GraphiQL playground at `/graphql`) |
| gRPC     | 50051 | High-performance, typed clients, service-to-service |

## Getting Started

### Prerequisites

- **Rust** (edition 2024) — install via [rustup](https://rustup.rs/)
- **tmux** — `brew install tmux` / `apt install tmux`
- **protoc** — Protocol Buffers compiler (`brew install protobuf` / `apt install protobuf-compiler`)

### Build & Run

```bash
cargo build
cargo run
```

The server starts two listeners:
- **HTTP** (REST + GraphQL) on `http://localhost:8080`
- **gRPC** on `localhost:50051`

### Configuration

Logging is controlled via the `RUST_LOG` environment variable:

```bash
RUST_LOG=tmux_gateway=debug cargo run
```

## API Quick Reference

### REST

```bash
# Health check
curl http://localhost:8080/health

# Hello
curl http://localhost:8080/hello/world
```

### GraphQL

Open the interactive GraphiQL playground at `http://localhost:8080/graphql`, or query directly:

```bash
curl -X POST http://localhost:8080/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "{ health }"}'
```

### gRPC

Using [grpcurl](https://github.com/fullstorydev/grpcurl):

```bash
grpcurl -plaintext localhost:50051 health.Health/Check
```

## Project Structure

```
tmux-gateway/
├── proto/              # Protocol Buffer definitions
│   └── health.proto
├── src/
│   ├── main.rs         # Entrypoint — spawns HTTP & gRPC servers
│   ├── rest/           # Axum REST routes
│   ├── graphql/        # async-graphql schema & handler
│   └── grpc/           # tonic gRPC service implementations
├── build.rs            # Protobuf code generation
└── Cargo.toml
```

## Tech Stack

- [Axum](https://github.com/tokio-rs/axum) — HTTP framework (REST + GraphQL serving)
- [async-graphql](https://github.com/async-graphql/async-graphql) — GraphQL server
- [Tonic](https://github.com/hyperium/tonic) — gRPC framework
- [Tokio](https://tokio.rs/) — Async runtime
- [Prost](https://github.com/tokio-rs/prost) — Protobuf serialization

## License

MIT
