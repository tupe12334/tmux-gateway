---
displayNumber: 133
status: open
priority: 2
createdAt: 2026-03-14T15:15:00.000000+00:00
updatedAt: 2026-03-14T15:15:00.000000+00:00
---

# Add KeySequence value object for type-safe key input to send_keys

## Problem

`send_keys` accepts `keys: &[String]` — raw unvalidated strings. This has several issues:

1. Special tmux keys like `Enter`, `Escape`, `C-c`, `C-d`, `Up`, `Down` are passed as magic strings with no compile-time validation
2. Dangerous key sequences (e.g., `C-c` to kill a process, `C-d` to close a shell) have no domain-level distinction from safe keys
3. No way to build key sequences programmatically with safety guarantees
4. Issue #85 calls for send_keys input validation but there's no domain type to validate against

## Proposed change

Add a `KeySequence` value object and `Key` enum in `tmux-gateway-core`:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Key {
    Literal(String),       // Regular text input
    Enter,
    Escape,
    Tab,
    Space,
    Backspace,
    Up, Down, Left, Right,
    Control(char),         // C-a, C-c, C-d, etc.
    Meta(char),            // M-a, M-b, etc.
    Function(u8),          // F1-F12
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeySequence {
    keys: Vec<Key>,
}

impl KeySequence {
    pub fn new(keys: Vec<Key>) -> Result<Self, ValidationError>;
    pub fn from_raw_strings(raw: &[String]) -> Result<Self, ValidationError>;
    pub fn to_tmux_args(&self) -> Vec<String>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn contains_control_key(&self) -> bool;
}
```

Pure helper methods:
- `from_raw_strings()` — parse raw API input into structured keys
- `to_tmux_args()` — serialize back to tmux-compatible format
- `contains_control_key()` — check if sequence includes potentially dangerous control keys

## Why this matters

- Foundation for issue #85 (send_keys input validation)
- Enables API layers to document available key types in schemas (GraphQL enum, protobuf enum, OpenAPI enum)
- Pure type with no I/O — perfect functional core candidate
- Prevents misspelled key names from reaching tmux silently

## Acceptance criteria

- [ ] `Key` enum with variants for all common tmux key names
- [ ] `KeySequence` value object with validated constructor
- [ ] `from_raw_strings()` parser for API input
- [ ] `to_tmux_args()` serializer for tmux command building
- [ ] Unit tests for parsing, round-trip, invalid input rejection
- [ ] `send_keys` updated to accept `KeySequence` instead of `&[String]`
