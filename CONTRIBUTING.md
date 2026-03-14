# Contributing to tmux-gateway

Thanks for your interest in contributing! This guide will help you get started.

## Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain, edition 2024)
- [tmux](https://github.com/tmux/tmux) installed locally (required for tests)
- [pnpm](https://pnpm.io/) (for git hooks)
- [protoc](https://grpc.io/docs/protoc-installation/) (Protocol Buffers compiler, needed for schema generation)

Optional but recommended:

- [Docker](https://docs.docker.com/get-docker/) (for containerized builds)
- `cargo-deny`, `cargo-audit`, `cargo-machete`, `cargo-tarpaulin` (for the full `make check` suite)
- [cspell](https://cspell.org/) (spell checking)

## Getting Started

```bash
# Clone the repository
git clone https://github.com/tupe12334/tmux-gateway.git
cd tmux-gateway

# Install git hooks
pnpm install

# Build (compiles + exports API schemas)
make build

# Run tests
make test

# Run the server
make run
```

The server exposes:

- **REST + GraphQL** on port `8080`
- **gRPC** on port `50051`

## Project Structure

```
tmux-gateway/
├── crates/tmux-gateway-core/   # Core domain logic (no networking)
├── src/
│   ├── api/
│   │   ├── rest/               # Axum REST routes + OpenAPI
│   │   ├── graphql/            # async-graphql schema
│   │   └── grpc/               # Tonic gRPC service
│   └── main.rs                 # Server entrypoint
├── schemas/                    # Generated API schemas (committed)
├── Makefile                    # Build orchestration
└── Dockerfile                  # Multi-stage container build
```

## Development Workflow

### Building

```bash
make build    # Compile and regenerate schemas
make schemas  # Regenerate schemas only
```

Schemas (`openapi.json`, `schema.graphql`, `tmux_gateway.proto`) are generated from code and committed to `schemas/`. CI will fail if they drift from the source.

### Testing

```bash
make test     # Run all workspace tests
```

tmux must be installed for tests to pass.

### Linting & Formatting

```bash
make lint     # cargo fmt --check + cargo clippy -D warnings
make check    # Full suite: lint + test + docs + deny + audit + machete + cspell
```

- **Formatting**: `cargo fmt` (default settings)
- **Linting**: `cargo clippy` with warnings treated as errors
- **Spell check**: cspell — add project-specific words to `cspell.json`

### Docker

```bash
make docker-up    # Build and start containers
make docker-down  # Stop containers
```

## Submitting Changes

1. Fork the repo and create a feature branch from `main`.
2. Make your changes, keeping commits focused and well-described.
3. Ensure `make check` passes locally.
4. Verify generated schemas are up to date (`make build`, then check for uncommitted changes in `schemas/`).
5. Open a pull request against `main`.

### CI Checks

PRs are validated by the following CI jobs:

| Check | What it does |
|-------|-------------|
| **build** | `cargo build --release` + `cargo test --release` |
| **fmt** | `cargo fmt --check` |
| **clippy** | `cargo clippy --workspace -- -D warnings` |
| **schema-drift** | Ensures committed schemas match generated output |
| **deny** | License and advisory compliance |
| **audit** | Security vulnerability scan |
| **machete** | Unused dependency detection |
| **dylint** | Custom lint rules (requires nightly) |
| **cspell** | Spell checking |
| **docker-build** | Dockerfile builds successfully |
| **coverage** | Test coverage report |

All checks must pass before a PR can be merged.

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
