---
displayNumber: 100
status: open
priority: 2
createdAt: 2026-03-14T14:38:59.416813+00:00
updatedAt: 2026-03-14T14:38:59.416813+00:00
---

# Add source-file domain operation for applying tmux configuration

tmux \`source-file\` loads and executes commands from a configuration file, allowing the gateway to apply tmux configurations programmatically. This is essential for production deployments where tmux needs consistent configuration across server restarts.

## Problem
- No way to apply tmux configuration through the gateway
- Operators must SSH in and manually source configs
- The declarative session spec (#81) creates sessions but doesn't handle global tmux configuration (key bindings, options, status bar)
- Configuration management is a core operations concern missing from the domain

## Proposed solution

\`\`\`rust
/// Execute commands from a tmux configuration file.
/// The file must exist and be readable by the tmux process.
pub async fn source_file(path: &str) -> Result<(), TmuxError>
\`\`\`

Validation:
- Path must be an absolute path (no relative traversal)
- Path must not contain shell metacharacters
- Reject paths outside allowed directories (configurable allowlist)

Security: This operation executes arbitrary tmux commands from a file, so API layers should gate it with appropriate authorization.

## Scope
- New \`source_file.rs\` module in \`tmux-gateway-core\`
- Path validation in \`validation.rs\`
- Add to \`TmuxCommands\` trait
- Complements #81 (declarative sessions) and #59/#87 (option management)
