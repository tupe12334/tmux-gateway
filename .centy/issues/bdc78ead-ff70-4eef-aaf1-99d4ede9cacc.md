---
displayNumber: 172
status: open
priority: 3
createdAt: 2026-03-15T10:30:00.000000+00:00
updatedAt: 2026-03-15T10:30:00.000000+00:00
---

# Add clock mode domain operation for pane time display

## Problem

Tmux has a built-in clock display mode (`clock-mode` command, bound to `Ctrl-b t` by default) that shows a large clock in a pane. While niche, it demonstrates tmux's modal pane behavior — panes can enter special modes (clock, copy, etc.) that override their normal content.

The gateway has no operation for triggering clock mode, and more importantly, has no domain concept of "pane mode" — whether a pane is in normal, copy, or clock mode.

## Proposed change

Add a clock mode operation and pane mode query:

```rust
/// The current mode of a tmux pane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaneMode {
    Normal,
    Copy,
    Clock,
    View,
}

/// Enter clock mode in the target pane.
pub async fn enter_clock_mode(
    executor: &impl TmuxExecutor,
    target: &str,
) -> Result<(), TmuxError> {
    // tmux clock-mode -t {target}
}

/// Query the current mode of a pane.
pub async fn get_pane_mode(
    executor: &impl TmuxExecutor,
    target: &str,
) -> Result<PaneMode, TmuxError> {
    // Query #{pane_mode} format variable
}
```

Also add `mode: PaneMode` to `TmuxPane` for inclusion in list_panes results.

## Why this matters

- Introduces the concept of pane modes to the domain model — a gap that affects copy mode (#78), view mode, and clock mode
- `PaneMode` enum is a value object that captures an important aspect of pane state
- Knowing a pane's mode is essential for automation: don't send keys to a pane in copy mode
- Complements `list_panes` with runtime state information beyond static metadata
- API-agnostic: mode information is useful regardless of API transport

## Acceptance criteria

- [ ] `PaneMode` enum with Normal, Copy, Clock, View variants
- [ ] `enter_clock_mode(executor, target)` via `tmux clock-mode`
- [ ] `get_pane_mode(executor, target)` queries `#{pane_mode}`
- [ ] `TmuxPane` includes `mode: PaneMode` field
- [ ] Parse `pane_mode` format output into enum
- [ ] Target validation
- [ ] Unit tests with mock executor
