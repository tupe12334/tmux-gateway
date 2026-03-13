use async_graphql::{EmptySubscription, Object, Schema, SimpleObject};

use crate::tmux::{self, TmuxCommands};

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

#[derive(SimpleObject)]
struct Session {
    name: String,
    windows: u32,
    created: String,
    attached: bool,
}

struct GraphqlHandler;

impl TmuxCommands for GraphqlHandler {
    async fn ls(&self) -> Result<Vec<tmux::TmuxSession>, String> {
        tmux::list_sessions().await
    }

    async fn new_session(&self, name: &str) -> Result<String, String> {
        tmux::new_session(name).await
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
            .map_err(async_graphql::Error::new)?;

        Ok(sessions
            .into_iter()
            .map(|s| Session {
                name: s.name,
                windows: s.windows,
                created: s.created,
                attached: s.attached,
            })
            .collect())
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn new_session(&self, name: String) -> async_graphql::Result<String> {
        GraphqlHandler
            .new_session(&name)
            .await
            .map_err(async_graphql::Error::new)
    }
}

pub fn build_schema() -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish()
}
