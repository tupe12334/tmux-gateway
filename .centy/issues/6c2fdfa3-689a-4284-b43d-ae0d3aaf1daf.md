---
displayNumber: 181
status: open
priority: 3
createdAt: 2026-03-16T05:04:05.638170+00:00
updatedAt: 2026-03-16T05:04:05.638170+00:00
---

# Add event-driven session example

# Add event-driven session example

## Problem
No example shows how to use the event system for reactive tmux workflows.

## Proposed change
Add `examples/events.rs` demonstrating:
- Create a session with events via `new_session_with_events`
- Listen for tmux events using EventReceiver
- React to session/window/pane changes
- Cleanup

## Acceptance criteria
- [ ] `examples/events.rs` exists and compiles
- [ ] Runs with `cargo run --example events`
- [ ] Demonstrates the event sender/receiver pattern
