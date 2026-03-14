---
# This file is managed by Centy. Use the Centy CLI to modify it.
displayNumber: 158
status: in-progress
priority: 2
createdAt: 2026-03-15T10:00:00.000000+00:00
updatedAt: 2026-03-14T17:21:54.375254+00:00
---

# Add pane zoom toggle domain operation

## Problem

Tmux supports zooming a pane to fill the entire window (`resize-pane -Z`), a heavily used feature for focusing on a single pane’s content. The gateway currently has no domain operation for this, meaning clients must use `send_keys` workarounds or cannot zoom at all.

This is a core tmux primitive alongside `select_pane`, `split_window`, and `swap_panes` — its absence creates a gap in the pane management surface.

## Proposed change

Add a `zoom_pane` domain operation in `tmux-gateway-core`:

````rust
/// Toggle zoom state on the target pane.
/// When zoomed, the pane fills the entire window area.
/// Calling again on a zoomed pane unzooms it.
pub async fn zoom_pane(executor: &impl TmuxExecutor, target: &str) -> Result<(), TmuxError> {
    // Validates target, then executes: tmux resize-pane -Z -t {target}
}

/// Check whether a pane is currently zoomed.
pub async fn is_pane_zoomed(executor: &impl TmuxExecutor, target: &str) -> Result<bool, TmuxError> {
    // Queries pane format: #{window_zoomed_flag}
}
````

Also add a `zoomed: bool` field to `TmuxPane` (populated from `#{window_zoomed_flag}` in `list_panes`).

## Why this matters

* Completes the pane lifecycle operations (create, select, split, swap, kill — but no zoom)
* Zoom is essential for automation: focus a pane before capturing its full-window output
* The `is_pane_zoomed` query enables conditional logic in orchestration workflows
* API-agnostic: REST, gRPC, and GraphQL all benefit from the same domain operation

## Acceptance criteria

* [ ] `zoom_pane(executor, target)` toggles pane zoom state via `resize-pane -Z`
* [ ] `is_pane_zoomed(executor, target)` returns zoom state
* [ ] `TmuxPane` struct includes `zoomed: bool` field
* [ ] Target validation applied before execution
* [ ] Unit tests with mock executor verify correct tmux args
* [ ] Error mapping for invalid/missing pane targets
