#![allow(unknown_lints, no_wrapper_functions)]

use async_graphql::{Enum, Object, Schema, SimpleObject, Subscription};
use chrono::{DateTime, Utc};
use std::time::Duration;

use crate::tmux::{self, RealTmuxExecutor, TmuxCommands};

pub type AppSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

/// A tmux session.
/// See: https://man.openbsd.org/tmux#CLIENTS_AND_SESSIONS
#[derive(SimpleObject)]
struct Session {
    id: String,
    name: String,
    windows: u32,
    created: String,
    attached: bool,
}

/// A tmux window.
/// See: https://man.openbsd.org/tmux#WINDOWS_AND_PANES
#[derive(SimpleObject)]
struct Window {
    id: String,
    index: u32,
    name: String,
    panes: u32,
    active: bool,
}

/// A tmux pane.
/// See: https://man.openbsd.org/tmux#WINDOWS_AND_PANES
#[derive(SimpleObject)]
struct Pane {
    id: String,
    width: u32,
    height: u32,
    active: bool,
    current_path: String,
    current_command: String,
    pid: u32,
}

/// Scope at which a tmux option is set.
/// See: https://man.openbsd.org/tmux#OPTIONS
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
enum GqlOptionScope {
    Global,
    Session,
    Window,
}

impl From<GqlOptionScope> for tmux::OptionScope {
    fn from(s: GqlOptionScope) -> Self {
        match s {
            GqlOptionScope::Global => tmux::OptionScope::Global,
            GqlOptionScope::Session => tmux::OptionScope::Session,
            GqlOptionScope::Window => tmux::OptionScope::Window,
        }
    }
}

impl From<tmux::OptionScope> for GqlOptionScope {
    fn from(s: tmux::OptionScope) -> Self {
        match s {
            tmux::OptionScope::Global => GqlOptionScope::Global,
            tmux::OptionScope::Session => GqlOptionScope::Session,
            tmux::OptionScope::Window => GqlOptionScope::Window,
        }
    }
}

/// A tmux option (name-value pair with scope).
/// See: https://man.openbsd.org/tmux#OPTIONS
#[derive(SimpleObject)]
struct TmuxOptionGql {
    name: String,
    value: String,
    scope: GqlOptionScope,
}

struct GraphqlHandler;

impl TmuxCommands for GraphqlHandler {
    fn executor(&self) -> RealTmuxExecutor {
        RealTmuxExecutor::new()
    }
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn health(&self) -> &str {
        "healthy"
    }

    /// List all tmux sessions.
    /// See: https://man.openbsd.org/tmux#list-sessions
    async fn ls(&self) -> async_graphql::Result<Vec<Session>> {
        let sessions = GraphqlHandler
            .ls()
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(sessions
            .into_iter()
            .map(|s| Session {
                id: s.id,
                name: s.name,
                windows: s.windows,
                created: DateTime::<Utc>::from_timestamp(s.created, 0)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_else(|| s.created.to_string()),
                attached: s.attached,
            })
            .collect())
    }

    /// List windows in a session.
    /// See: https://man.openbsd.org/tmux#list-windows
    async fn list_windows(&self, session: String) -> async_graphql::Result<Vec<Window>> {
        let windows = GraphqlHandler
            .list_windows(&session)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(windows
            .into_iter()
            .map(|w| Window {
                id: w.id,
                index: w.index,
                name: w.name,
                panes: w.panes,
                active: w.active,
            })
            .collect())
    }

    /// List panes in a window.
    /// See: https://man.openbsd.org/tmux#list-panes
    async fn list_panes(&self, target: String) -> async_graphql::Result<Vec<Pane>> {
        let panes = GraphqlHandler
            .list_panes(&target)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(panes
            .into_iter()
            .map(|p| Pane {
                id: p.id,
                width: p.width,
                height: p.height,
                active: p.active,
                current_path: p.current_path,
                current_command: p.current_command,
                pid: p.pid,
            })
            .collect())
    }

    /// Capture the visible contents of a pane.
    /// See: https://man.openbsd.org/tmux#capture-pane
    async fn capture_pane(&self, target: String) -> async_graphql::Result<String> {
        GraphqlHandler
            .capture_pane(&target)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    /// Capture pane contents with advanced options.
    /// See: https://man.openbsd.org/tmux#capture-pane
    async fn capture_pane_with_options(
        &self,
        target: String,
        start_line: Option<i32>,
        end_line: Option<i32>,
        #[graphql(default = false)] escape_sequences: bool,
    ) -> async_graphql::Result<String> {
        let opts = tmux::CaptureOptions {
            start_line,
            end_line,
            escape_sequences,
        };
        GraphqlHandler
            .capture_pane_with_options(&target, &opts)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
    }

    /// Get a tmux option value.
    /// See: https://man.openbsd.org/tmux#show-options
    async fn get_option(
        &self,
        name: String,
        scope: GqlOptionScope,
        #[graphql(default)] target: Option<String>,
    ) -> async_graphql::Result<String> {
        let opt = GraphqlHandler
            .get_option(&name, scope.into(), target.as_deref())
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(opt.value)
    }

    /// List tmux options for a scope.
    /// See: https://man.openbsd.org/tmux#show-options
    async fn list_options(
        &self,
        scope: GqlOptionScope,
        #[graphql(default)] target: Option<String>,
    ) -> async_graphql::Result<Vec<TmuxOptionGql>> {
        let options = GraphqlHandler
            .list_options(scope.into(), target.as_deref())
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(options
            .into_iter()
            .map(|o| TmuxOptionGql {
                name: o.name,
                value: o.value,
                scope: o.scope.into(),
            })
            .collect())
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Create a new tmux session.
    /// See: https://man.openbsd.org/tmux#new-session
    async fn create_session(
        &self,
        name: String,
        #[graphql(default)] command: Option<String>,
    ) -> async_graphql::Result<Session> {
        let s = GraphqlHandler
            .create_session(&name, command.as_deref())
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(Session {
            id: s.id,
            name: s.name,
            windows: s.windows,
            created: DateTime::<Utc>::from_timestamp(s.created, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| s.created.to_string()),
            attached: s.attached,
        })
    }

    /// Destroy a session.
    /// See: https://man.openbsd.org/tmux#kill-session
    async fn kill_session(&self, target: String) -> async_graphql::Result<bool> {
        GraphqlHandler
            .kill_session(&target)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }

    /// Destroy a window.
    /// See: https://man.openbsd.org/tmux#kill-window
    async fn kill_window(&self, target: String) -> async_graphql::Result<bool> {
        GraphqlHandler
            .kill_window(&target)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }

    /// Destroy a pane.
    /// See: https://man.openbsd.org/tmux#kill-pane
    async fn kill_pane(&self, target: String) -> async_graphql::Result<bool> {
        GraphqlHandler
            .kill_pane(&target)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }

    /// Send key(s) to a pane.
    /// See: https://man.openbsd.org/tmux#send-keys
    async fn send_keys(&self, target: String, keys: Vec<String>) -> async_graphql::Result<bool> {
        GraphqlHandler
            .send_keys(&target, &keys)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }

    /// Rename a session.
    /// See: https://man.openbsd.org/tmux#rename-session
    async fn rename_session(
        &self,
        target: String,
        new_name: String,
    ) -> async_graphql::Result<bool> {
        GraphqlHandler
            .rename_session(&target, &new_name)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }

    /// Rename a window.
    /// See: https://man.openbsd.org/tmux#rename-window
    async fn rename_window(&self, target: String, new_name: String) -> async_graphql::Result<bool> {
        GraphqlHandler
            .rename_window(&target, &new_name)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }

    /// Create a new window in a session.
    /// See: https://man.openbsd.org/tmux#new-window
    async fn new_window(
        &self,
        session: String,
        name: String,
        #[graphql(default)] command: Option<String>,
    ) -> async_graphql::Result<Window> {
        let w = GraphqlHandler
            .new_window(&session, &name, command.as_deref())
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(Window {
            id: w.id,
            index: w.index,
            name: w.name,
            panes: w.panes,
            active: w.active,
        })
    }

    /// Split a pane to create a new pane.
    /// See: https://man.openbsd.org/tmux#split-window
    async fn split_window(
        &self,
        target: String,
        horizontal: bool,
        #[graphql(default)] command: Option<String>,
    ) -> async_graphql::Result<Pane> {
        let p = GraphqlHandler
            .split_window(&target, horizontal, command.as_deref())
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(Pane {
            id: p.id,
            width: p.width,
            height: p.height,
            active: p.active,
            current_path: p.current_path,
            current_command: p.current_command,
            pid: p.pid,
        })
    }

    /// Create a session with multiple named windows.
    /// See: https://man.openbsd.org/tmux#new-session
    async fn create_session_with_windows(
        &self,
        name: String,
        window_names: Vec<String>,
    ) -> async_graphql::Result<Session> {
        let session = GraphqlHandler
            .create_session_with_windows(&name, &window_names)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(Session {
            id: session.id,
            name: session.name,
            windows: session.windows,
            created: DateTime::<Utc>::from_timestamp(session.created, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| session.created.to_string()),
            attached: session.attached,
        })
    }

    /// Swap two panes.
    /// See: https://man.openbsd.org/tmux#swap-pane
    async fn swap_panes(&self, src: String, dst: String) -> async_graphql::Result<bool> {
        GraphqlHandler
            .swap_panes(&src, &dst)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }

    /// Move a window to another session.
    /// See: https://man.openbsd.org/tmux#move-window
    async fn move_window(
        &self,
        source: String,
        destination_session: String,
    ) -> async_graphql::Result<bool> {
        GraphqlHandler
            .move_window(&source, &destination_session)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }

    /// Select (activate) a window.
    /// See: https://man.openbsd.org/tmux#select-window
    async fn select_window(&self, target: String) -> async_graphql::Result<bool> {
        GraphqlHandler
            .select_window(&target)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }

    /// Select (activate) a pane.
    /// See: https://man.openbsd.org/tmux#select-pane
    async fn select_pane(&self, target: String) -> async_graphql::Result<bool> {
        GraphqlHandler
            .select_pane(&target)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }

    /// Apply a layout to a window.
    /// See: https://man.openbsd.org/tmux#select-layout
    async fn select_layout(&self, target: String, layout: String) -> async_graphql::Result<bool> {
        let l = match layout.as_str() {
            "even-horizontal" => tmux::PaneLayout::EvenHorizontal,
            "even-vertical" => tmux::PaneLayout::EvenVertical,
            "main-horizontal" => tmux::PaneLayout::MainHorizontal,
            "main-vertical" => tmux::PaneLayout::MainVertical,
            "tiled" => tmux::PaneLayout::Tiled,
            "" => {
                return Err(async_graphql::Error::new("layout must not be empty"));
            }
            other => tmux::PaneLayout::Custom(other.to_string()),
        };
        GraphqlHandler
            .select_layout(&target, l)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }

    /// Resize a pane.
    /// See: https://man.openbsd.org/tmux#resize-pane
    async fn resize_pane(
        &self,
        target: String,
        direction: String,
        amount: u32,
    ) -> async_graphql::Result<bool> {
        let dir = match direction.as_str() {
            "up" | "Up" | "U" => tmux::ResizeDirection::Up(amount),
            "down" | "Down" | "D" => tmux::ResizeDirection::Down(amount),
            "left" | "Left" | "L" => tmux::ResizeDirection::Left(amount),
            "right" | "Right" | "R" => tmux::ResizeDirection::Right(amount),
            _ => {
                return Err(async_graphql::Error::new(format!(
                    "invalid direction: {direction}"
                )));
            }
        };
        GraphqlHandler
            .resize_pane(&target, dir)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }

    /// Set a tmux option.
    /// See: https://man.openbsd.org/tmux#set-option
    async fn set_option(
        &self,
        name: String,
        value: String,
        scope: GqlOptionScope,
        #[graphql(default)] target: Option<String>,
    ) -> async_graphql::Result<bool> {
        GraphqlHandler
            .set_option(&name, &value, scope.into(), target.as_deref())
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }
}

#[derive(SimpleObject)]
struct PaneOutputEvent {
    content: String,
    timestamp: String,
}

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn pane_output(
        &self,
        target: String,
        #[graphql(default = 500)] interval_ms: i32,
    ) -> impl futures_core::Stream<Item = PaneOutputEvent> {
        let interval = Duration::from_millis((interval_ms as u64).clamp(100, 10000));

        async_stream::stream! {
            let Ok(target) = tmux::PaneTarget::try_from(target.as_str()) else {
                return;
            };
            let mut last_content = String::new();
            let mut ticker = tokio::time::interval(interval);

            loop {
                ticker.tick().await;
                match tmux::capture_pane(&RealTmuxExecutor::new(), &target).await {
                    Ok(content) => {
                        if content != last_content {
                            last_content = content.clone();
                            yield PaneOutputEvent {
                                content,
                                timestamp: Utc::now().to_rfc3339(),
                            };
                        }
                    }
                    Err(_) => break,
                }
            }
        }
    }
}

pub fn build_schema() -> AppSchema {
    let max_depth = std::env::var("GRAPHQL_MAX_DEPTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(15);
    let max_complexity = std::env::var("GRAPHQL_MAX_COMPLEXITY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(500);
    let introspection = std::env::var("GRAPHQL_INTROSPECTION")
        .map(|v| v != "false")
        .unwrap_or(true);

    let mut builder = Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .limit_depth(max_depth)
        .limit_complexity(max_complexity);

    if !introspection {
        builder = builder.disable_introspection();
    }

    builder.finish()
}
