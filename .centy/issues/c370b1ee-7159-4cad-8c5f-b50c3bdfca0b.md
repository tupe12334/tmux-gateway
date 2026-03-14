---
# This file is managed by Centy. Use the Centy CLI to modify it.
displayNumber: 35
status: in-progress
priority: 1
createdAt: 2026-03-14T13:18:19.483037+00:00
updatedAt: 2026-03-14T15:07:45.583185+00:00
---

# Move transport-specific error mapping out of domain TmuxError

TmuxError in tmux-gateway-core has methods and types that leak transport concerns into the domain layer, violating functional core / imperative shell separation.

## Problem (crates/tmux-gateway-core/src/error.rs):

* `http_status_code(&self) -> u16` maps domain errors to HTTP status codes
* `grpc_code(&self) -> GrpcCode` maps domain errors to gRPC codes
* `GrpcCode` enum (NotFound, InvalidArgument, Internal) is a transport concept defined in the domain crate

The domain core should be pure and transport-agnostic. HTTP status codes and gRPC codes are API layer concerns.

## Fix:

* Remove `http_status_code()` and `grpc_code()` from TmuxError
* Move `GrpcCode` to the gRPC API layer
* Each API layer maps TmuxError variants to its own transport codes (match on variants directly)
* This is a straightforward refactor since each API layer already has error-mapping functions
