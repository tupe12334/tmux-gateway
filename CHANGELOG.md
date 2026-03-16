# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2026-03-16

### Added

- Working directory and multi-arg command support to `new_session`
- `SessionLock` for per-session mutation serialization
- `kill-server` domain operation for full tmux server shutdown
- `show-messages` domain operation for tmux server message log access
- `respawn-pane` and `respawn-window` domain operations
- `has_session` domain query for O(1) session existence checks
- Domain precondition checks before destructive and mutating operations
- Domain-level `OperationTimeout` with configurable per-operation deadlines
- `ErrorRecoverability` enum for domain error retry guidance
- Validated newtypes for domain identifiers (`SessionName`, `WindowTarget`, `PaneTarget`)
- Declarative session specification value objects and `apply_session_spec`
- Pane target validation to `capture_pane`
- `tmux_docs` module for schema enrichment with tmux documentation links
- Auto-generated AsyncAPI 3.0 spec for async messaging interfaces
- Injectable tmux server socket path in domain operations
- Shutdown integration tests
- Commitlint with conventional commits and husky commit-msg hook
- Rust examples: basic session lifecycle, window/pane management, send keys and capture output

### Changed

- Separated `TmuxExecutor` port trait from `RealTmuxExecutor` adapter
- Extracted pure validate-build-parse pipeline for each domain operation
- Release binary optimizations: LTO, strip debug symbols, `opt-level = "z"`, `codegen-units = 1`

### Fixed

- Clippy warnings: duplicated_attributes, thin-wrapper, collapsible-if lint errors
- Formatting and import ordering across codebase

## [0.1.1] - 2026-03-16

### Added

- WebSocket support for real-time pane output streaming
- Unix socket support for both HTTP and gRPC servers
- Domain-level structured logging via `LogPort` trait
- Domain-level pagination for list operations
- Domain-level idempotency support for create operations
- Domain event types for operation notifications
- Domain-level tracing spans to all core operations
- Domain health model with component-level readiness checks
- Hierarchical domain model with `SessionDetail` and `WindowDetail` aggregates
- Composite domain operations for multi-step tmux workflows
- Domain operations for tmux session options and server-level environment variables
- Stable tmux IDs (`$id` / `@id`) on `TmuxSession` and `TmuxWindow` domain models
- `pid`, `current_path`, and `current_command` fields on `TmuxPane` domain model
- `Display` implementations for `TmuxSession`, `TmuxWindow`, and `TmuxPane`
- `PartialEq` and `Eq` derives on domain model types
- Default `TmuxCommands` implementation to reduce API boilerplate
- Typed `i64` timestamp for `TmuxSession.created`
- `TmuxServerInfo` domain type for tmux health and version
- Domain-level session existence check (`has_session`)
- Select window, select pane, swap window, and resize pane domain operations
- Layout management domain operations
- Max-length validation for target identifier strings
- Capture options for scroll history and line ranges
- Normalized `capture_pane` output in domain layer
- Optional command parameter to `new_session`, `new_window`, `split_window`
- `Validation` variant on `TmuxError` to preserve validation error context
- `SessionAlreadyExists` error variant for duplicate session detection
- Return richer types from mutating domain operations
- `LogPort` trait for domain-level structured logging
- SHA256 checksums to release workflow
- Rust examples: batch session setup, options management, event-driven session
- WebSocket documentation to README

### Changed

- Extracted CORS layer building into dedicated `cors` module
- Extracted transport layer into `tcp/{http,grpc}` and `unix/{http,grpc}` modules
- Separated tmux I/O adapter from domain operations
- Extracted tmux output parsing into pure functions
- Moved transport-specific error mapping out of domain `TmuxError`
- Removed `serde::Serialize` from domain model types
- Removed `pub` re-export of `tmux_interface` from core crate API
- Consolidated `spawn_blocking` boilerplate with executor helper
- Gated schema export behind `EXPORT_SCHEMAS` env var
- Merged `.env.dev` into `.env` to use a single env file
- Pinned Rust toolchain with `rust-toolchain.toml`

### Fixed

- CORS default origins to use frontend dev port instead of gRPC port
- Warn on invalid CORS origins and fail if none are valid
- Silent parsing failures in list operations now return `TmuxError::ParseError`
- `capture_pane` handler to use dedicated `CapturePaneRequest` type
- GraphQL default depth/complexity limits increased to fix introspection
- gRPC TOCTOU race condition in port binding
- Clippy and formatting warnings across codebase

## [0.1.0] - 2026-03-15

Initial release.

### Added

- REST API on port 8080 with Swagger UI and OpenAPI schema
- GraphQL API on port 8080 with GraphiQL playground
- gRPC API on port 50051 with reflection and health service
- Core tmux operations: list sessions, create session, kill session, kill window, kill pane
- Expanded tmux operations: list windows, list panes, send keys, rename session, rename window, new window, split window, capture pane
- Per-IP rate limiting middleware
- gRPC request logging and observability middleware
- Request body size limits to prevent DoS attacks
- Input validation and sanitization for all user inputs
- Graceful shutdown with signal handling
- `TmuxError` custom error type replacing string errors
- Timeout for tmux operations to prevent thread pool exhaustion
- GraphQL query depth and complexity limits
- gRPC health check that verifies tmux availability periodically
- Effective configuration logging at startup
- Guard-based session cleanup for integration tests
- Comprehensive test suite: REST, gRPC, GraphQL, and E2E integration tests
- Docker support with `Dockerfile` and `docker-compose.yml`
- Non-root user and `HEALTHCHECK` in Dockerfile
- CI/CD GitHub Actions for build and release
- CI checks: fmt, deny, audit, machete, cspell
- Test job in release workflow
- Schema export validation in pre-push hook
- Code-first gRPC proto generation
- `tmux-gateway-core` crate for domain logic
- Mermaid architecture diagram in README
- `Makefile` with test, lint, and check targets
- MIT license

[0.1.2]: https://github.com/tupe12334/tmux-gateway/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/tupe12334/tmux-gateway/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/tupe12334/tmux-gateway/releases/tag/v0.1.0
