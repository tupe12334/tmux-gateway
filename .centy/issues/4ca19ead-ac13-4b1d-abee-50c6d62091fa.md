---
displayNumber: 59
status: open
priority: 2
createdAt: 2026-03-14T13:27:50.237117+00:00
updatedAt: 2026-03-14T13:27:50.237117+00:00
---

# Add domain-level size limit for capture_pane output

capture_pane returns the entire pane content as an unbounded String. A pane with scrollback history could produce megabytes of output, causing memory pressure or DoS.

## Problem
- capture_pane collects all stdout from tmux capture-pane with no size limit
- A pane with large scrollback buffer (thousands of lines) produces proportionally large responses
- No domain-level protection against excessive output size
- All three API layers inherit this unbounded behavior

## What to do
Add an optional max_lines or max_bytes parameter to the domain function:

pub async fn capture_pane(target: &str, options: CaptureOptions) -> Result<String, TmuxError>

pub struct CaptureOptions {
    pub max_lines: Option<u32>,  // maps to tmux -S/-E flags
    pub start_line: Option<i32>, // negative = from scrollback
}

The domain enforces a sensible default maximum (e.g., 10000 lines) to prevent unbounded responses. API layers can expose these options as query parameters.

## Acceptance criteria
- capture_pane has a configurable line limit with a safe default
- tmux -S/-E flags are used to limit output at the source
- Domain function documents the maximum output size
