---
displayNumber: 101
status: open
priority: 2
createdAt: 2026-03-14T14:40:54.518856+00:00
updatedAt: 2026-03-14T14:40:54.518856+00:00
---

# Add clear-history domain operation for pane scrollback management

tmux \`clear-history\` deletes the scrollback buffer for a pane. For long-running terminals managed through the gateway, scrollback can grow unbounded, consuming memory. There is no domain operation to manage this.

## Problem
- Long-running panes accumulate unbounded scrollback history
- No way to reset or clear pane history through the gateway
- Memory-sensitive deployments need scrollback management
- Complements capture-pane (#51, #92) — capture before clearing for archival

## Proposed solution

\`\`\`rust
/// Clear the scrollback history for a pane.
pub async fn clear_history(target: &str) -> Result<(), TmuxError>
\`\`\`

- Validates pane target
- Uses tmux \`clear-history -t target\`
- Idempotent: clearing an already-empty history is a no-op

## Scope
- New \`clear_history.rs\` module in \`tmux-gateway-core\`
- Add to \`TmuxCommands\` trait
- Export from \`lib.rs\`
