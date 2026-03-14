---
displayNumber: 136
status: open
priority: 2
createdAt: 2026-03-14T15:12:45.692318+00:00
updatedAt: 2026-03-14T15:12:45.692318+00:00
---

# Add multi-server domain support for isolated tmux instance management

Issue #78 makes the tmux socket path injectable, but the domain still assumes a single tmux server. Production deployments often run multiple isolated tmux instances (e.g., one per environment, one per tenant, or one per security zone). The domain has no concept of server identity or multi-server coordination.

## Problem

- All domain operations implicitly target a single tmux server (default socket)
- No domain concept of server identity — operations cannot specify which server to target
- Multi-environment deployments must run separate gateway instances per tmux server
- Cannot compare or migrate sessions across servers
- #78 enables socket injection but doesn't model the multi-server pattern at the domain level

## Solution

Add a domain concept of server identity that all operations can reference:

```rust
pub struct TmuxServer {
    pub id: String,               // Logical server name (e.g., "production", "staging")
    pub socket_path: Option<String>, // tmux -S path (None = default)
    pub socket_name: Option<String>, // tmux -L name (None = default)
}

impl Default for TmuxServer {
    fn default() -> Self { Self { id: "default".to_string(), socket_path: None, socket_name: None } }
}

pub struct ServerRegistry {
    servers: HashMap<String, TmuxServer>,
}

impl ServerRegistry {
    pub fn get(&self, id: &str) -> Option<&TmuxServer>;
    pub fn list(&self) -> Vec<&TmuxServer>;
    pub fn register(&mut self, server: TmuxServer);
}
```

Domain operations accept an optional server context:

```rust
pub async fn list_sessions_on(server: &TmuxServer) -> Result<Vec<TmuxSession>, TmuxError>
pub async fn list_all_sessions(registry: &ServerRegistry) -> Result<HashMap<String, Vec<TmuxSession>>, TmuxError>
```

## Impact

- Enables single gateway instance to manage multiple tmux environments
- Foundation for cross-server session migration
- Server registry integrates with authorization (#113) for server-level access control
- All transport layers benefit from multi-server capability uniformly
