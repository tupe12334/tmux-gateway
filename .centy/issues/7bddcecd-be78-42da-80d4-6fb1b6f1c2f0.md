---
displayNumber: 50
status: open
priority: 1
createdAt: 2026-03-14T13:23:23.541054+00:00
updatedAt: 2026-03-14T13:23:23.541054+00:00
---

# Separate domain errors from transport-specific error mapping

TmuxError in the core crate contains methods like http_status_code() and grpc_code() that map domain errors to transport-specific codes. This couples the domain layer to API concerns.

## Problem
- TmuxError::http_status_code() returns HTTP status codes — HTTP is not a domain concept
- TmuxError::grpc_code() returns gRPC codes — gRPC is not a domain concept
- GrpcCode enum lives in the core crate but is only relevant to the gRPC API layer
- Adding a new transport (e.g., WebSocket) would require modifying the core error type

## What to do
- Remove http_status_code() and grpc_code() from TmuxError
- Remove GrpcCode enum from the core crate
- Each API layer implements its own From<TmuxError> mapping:
  - REST: impl From<TmuxError> for (StatusCode, String)
  - GraphQL: impl From<TmuxError> for async_graphql::Error
  - gRPC: impl From<TmuxError> for tonic::Status
- TmuxError stays focused on domain semantics: what went wrong, not how to report it

## Acceptance criteria
- Core crate has zero knowledge of HTTP, gRPC, or GraphQL
- Each API layer owns its error mapping logic
- Adding a new transport requires no changes to the core crate
