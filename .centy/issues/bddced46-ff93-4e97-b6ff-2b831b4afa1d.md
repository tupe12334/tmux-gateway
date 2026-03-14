---
displayNumber: 97
status: open
priority: 2
createdAt: 2026-03-14T14:38:43.508054+00:00
updatedAt: 2026-03-14T14:38:43.508054+00:00
---

# Add pipe-pane domain operation for pane output logging and streaming

tmux \`pipe-pane\` redirects a pane's output to a shell command or file, enabling persistent logging and output capture without polling. This is a powerful alternative to repeated capture-pane calls and complements the WebSocket streaming proposed in #8.

## Problem
- Real-time pane monitoring requires repeated capture-pane polling (wasteful)
- No way to persistently log pane output to a file through the gateway
- pipe-pane is a core tmux feature for output handling that has no domain representation
- Combined with capture-pane (#51, #75, #92), this completes the pane output management story

## Proposed solution

\`\`\`rust
/// Start piping pane output to a command or file.
/// If command is None, stops an existing pipe.
pub async fn pipe_pane(
    target: &str,
    command: Option<&str>,
) -> Result<(), TmuxError>

/// Check if a pane currently has an active pipe.
pub async fn is_pane_piped(target: &str) -> Result<bool, TmuxError>
\`\`\`

Design considerations:
- \`command\` is validated for safety (no shell metacharacters beyond what's needed)
- Passing \`None\` for command stops the current pipe (idempotent)
- The domain operation is API-agnostic: REST, GraphQL, and gRPC all express start/stop pipe
- This is a domain operation, not a transport concern — the pipe runs server-side in tmux

## Scope
- New \`pipe_pane.rs\` module in \`tmux-gateway-core\`
- Command validation in \`validation.rs\`
- Add to \`TmuxCommands\` trait
- Complements #8 (WebSocket streaming) and #51/#75/#92 (capture-pane improvements)
