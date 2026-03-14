use async_graphql::{EmptySubscription, Object, Schema, SimpleObject};
use chrono::{DateTime, Utc};

use crate::tmux::{self, TmuxCommands};

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

#[derive(SimpleObject)]
struct Session {
    name: String,
    windows: u32,
    created: String,
    attached: bool,
}

#[derive(SimpleObject)]
struct Window {
    index: u32,
    name: String,
    panes: u32,
    active: bool,
}

#[derive(SimpleObject)]
struct Pane {
    id: String,
    width: u32,
    height: u32,
    active: bool,
    current_path: String,
    current_command: String,
}

struct GraphqlHandler;

impl TmuxCommands for GraphqlHandler {}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn health(&self) -> &str {
        "healthy"
    }

    async fn ls(&self) -> async_graphql::Result<Vec<Session>> {
        let sessions = GraphqlHandler
            .ls()
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(sessions
            .into_iter()
            .map(|s| Session {
                name: s.name,
                windows: s.windows,
                created: DateTime::<Utc>::from_timestamp(s.created, 0)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_else(|| s.created.to_string()),
                attached: s.attached,
            })
            .collect())
    }

    async fn list_windows(&self, session: String) -> async_graphql::Result<Vec<Window>> {
        let windows = GraphqlHandler
            .list_windows(&session)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(windows
            .into_iter()
            .map(|w| Window {
                index: w.index,
                name: w.name,
                panes: w.panes,
                active: w.active,
            })
            .collect())
    }

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
            })
            .collect())
    }

    async fn capture_pane(&self, target: String) -> async_graphql::Result<String> {
        GraphqlHandler
            .capture_pane(&target)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))
    }

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
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_session(&self, name: String) -> async_graphql::Result<Session> {
        let s = GraphqlHandler
            .create_session(&name)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(Session {
            name: s.name,
            windows: s.windows,
            created: DateTime::<Utc>::from_timestamp(s.created, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| s.created.to_string()),
            attached: s.attached,
        })
    }

    async fn kill_session(&self, target: String) -> async_graphql::Result<bool> {
        GraphqlHandler
            .kill_session(&target)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }

    async fn kill_window(&self, target: String) -> async_graphql::Result<bool> {
        GraphqlHandler
            .kill_window(&target)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }

    async fn kill_pane(&self, target: String) -> async_graphql::Result<bool> {
        GraphqlHandler
            .kill_pane(&target)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }

    async fn send_keys(&self, target: String, keys: Vec<String>) -> async_graphql::Result<bool> {
        GraphqlHandler
            .send_keys(&target, &keys)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }

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

    async fn rename_window(&self, target: String, new_name: String) -> async_graphql::Result<bool> {
        GraphqlHandler
            .rename_window(&target, &new_name)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }

    async fn new_window(&self, session: String, name: String) -> async_graphql::Result<Window> {
        let w = GraphqlHandler
            .new_window(&session, &name)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(Window {
            index: w.index,
            name: w.name,
            panes: w.panes,
            active: w.active,
        })
    }

    async fn split_window(&self, target: String, horizontal: bool) -> async_graphql::Result<Pane> {
        let p = GraphqlHandler
            .split_window(&target, horizontal)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(Pane {
            id: p.id,
            width: p.width,
            height: p.height,
            active: p.active,
            current_path: p.current_path,
            current_command: p.current_command,
        })
    }

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
            name: session.name,
            windows: session.windows,
            created: DateTime::<Utc>::from_timestamp(session.created, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| session.created.to_string()),
            attached: session.attached,
        })
    }

    async fn swap_panes(&self, src: String, dst: String) -> async_graphql::Result<bool> {
        GraphqlHandler
            .swap_panes(&src, &dst)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }

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

    async fn select_window(&self, target: String) -> async_graphql::Result<bool> {
        GraphqlHandler
            .select_window(&target)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }

    async fn select_pane(&self, target: String) -> async_graphql::Result<bool> {
        GraphqlHandler
            .select_pane(&target)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }

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
}

pub fn build_schema() -> AppSchema {
    let max_depth = std::env::var("GRAPHQL_MAX_DEPTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    let max_complexity = std::env::var("GRAPHQL_MAX_COMPLEXITY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);
    let introspection = std::env::var("GRAPHQL_INTROSPECTION")
        .map(|v| v != "false")
        .unwrap_or(true);

    let mut builder = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .limit_depth(max_depth)
        .limit_complexity(max_complexity);

    if !introspection {
        builder = builder.disable_introspection();
    }

    builder.finish()
}
