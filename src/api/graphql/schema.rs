use async_graphql::{EmptySubscription, Object, Schema, SimpleObject};
use chrono::{DateTime, Utc};

use crate::tmux::{self, TmuxCommands, TmuxError};

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
}

struct GraphqlHandler;

impl TmuxCommands for GraphqlHandler {
    async fn ls(&self) -> Result<Vec<tmux::TmuxSession>, TmuxError> {
        tmux::list_sessions().await
    }

    async fn create_session(&self, name: &str) -> Result<tmux::TmuxSession, TmuxError> {
        tmux::new_session(name).await
    }

    async fn kill_session(&self, target: &str) -> Result<(), TmuxError> {
        tmux::kill_session(target).await
    }

    async fn kill_window(&self, target: &str) -> Result<(), TmuxError> {
        tmux::kill_window(target).await
    }

    async fn kill_pane(&self, target: &str) -> Result<(), TmuxError> {
        tmux::kill_pane(target).await
    }

    async fn list_windows(&self, session: &str) -> Result<Vec<tmux::TmuxWindow>, TmuxError> {
        tmux::list_windows(session).await
    }

    async fn list_panes(&self, target: &str) -> Result<Vec<tmux::TmuxPane>, TmuxError> {
        tmux::list_panes(target).await
    }

    async fn send_keys(&self, target: &str, keys: &[String]) -> Result<(), TmuxError> {
        tmux::send_keys(target, keys).await
    }

    async fn rename_session(&self, target: &str, new_name: &str) -> Result<(), TmuxError> {
        tmux::rename_session(target, new_name).await
    }

    async fn rename_window(&self, target: &str, new_name: &str) -> Result<(), TmuxError> {
        tmux::rename_window(target, new_name).await
    }

    async fn new_window(&self, session: &str, name: &str) -> Result<tmux::TmuxWindow, TmuxError> {
        tmux::new_window(session, name).await
    }

    async fn split_window(
        &self,
        target: &str,
        horizontal: bool,
    ) -> Result<tmux::TmuxPane, TmuxError> {
        tmux::split_window(target, horizontal).await
    }

    async fn capture_pane(&self, target: &str) -> Result<String, TmuxError> {
        tmux::capture_pane(target).await
    }
}

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
            })
            .collect())
    }

    async fn capture_pane(&self, target: String) -> async_graphql::Result<String> {
        GraphqlHandler
            .capture_pane(&target)
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
        })
    }
}

pub fn build_schema() -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish()
}
