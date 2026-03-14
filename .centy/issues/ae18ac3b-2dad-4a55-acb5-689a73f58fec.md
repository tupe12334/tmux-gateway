---
displayNumber: 132
status: open
priority: 3
createdAt: 2026-03-14T15:05:00.000000+00:00
updatedAt: 2026-03-14T15:05:00.000000+00:00
---

# Add hierarchical target resolution with fuzzy matching and suggestions

## Problem

When a user provides a target like `"myession:main"` (typo for `"mysession:main"`), the domain layer validates the format and passes it to tmux, which returns a cryptic "session not found" error. The user gets no help figuring out what went wrong.

The domain layer has all the information needed to provide helpful suggestions — it can list existing sessions and compute similarity — but currently doesn't.

## Proposed change

Add a `TargetResolver` module in `tmux-gateway-core` with pure resolution and suggestion logic:

```rust
/// Pure: parse a target string into its hierarchical components
pub struct ResolvedTarget {
    pub session: String,
    pub window: Option<String>,
    pub pane: Option<String>,
}

pub fn parse_target(target: &str) -> Result<ResolvedTarget, TmuxError>;

/// Pure: find the closest matching session name using edit distance
pub fn suggest_session(
    input: &str,
    existing_sessions: &[TmuxSession],
) -> Option<String>;

/// Pure: find the closest matching window name within a session
pub fn suggest_window(
    input: &str,
    existing_windows: &[TmuxWindow],
) -> Option<String>;

/// Pure: enrich a "not found" error with suggestions
pub fn enrich_not_found_error(
    error: TmuxError,
    sessions: &[TmuxSession],
    windows: Option<&[TmuxWindow]>,
) -> TmuxError;
```

Add a `TmuxError` variant for suggestion-enriched errors:

```rust
SessionNotFoundWithSuggestion { target: String, suggestion: String },
WindowNotFoundWithSuggestion { target: String, suggestion: String },
```

The edit distance computation is a pure function (Levenshtein or Jaro-Winkler). The imperative shell decides whether to fetch existing sessions for suggestion enrichment.

## Why this matters

- Better developer experience: "session 'myession' not found — did you mean 'mysession'?"
- Issue #82 (typed target value objects) provides the foundation; this adds intelligence on top
- Issue #20 (structured error responses) benefits from richer error content
- All suggestion logic is pure — no I/O, no tmux calls, fully testable

## Acceptance criteria

- [ ] `ResolvedTarget` struct with parse function
- [ ] Pure `suggest_session()` and `suggest_window()` using edit distance
- [ ] `enrich_not_found_error()` that adds suggestions when similarity is high enough
- [ ] Suggestion threshold: only suggest if edit distance ≤ 3 (configurable)
- [ ] Unit tests: exact match, close typo, no match, empty list, multiple candidates
- [ ] No external crate for edit distance — implement as pure function
