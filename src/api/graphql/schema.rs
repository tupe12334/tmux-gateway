use async_graphql::{EmptySubscription, Object, Schema, SimpleObject};

use crate::tmux;

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

#[derive(SimpleObject)]
struct Session {
    name: String,
    windows: u32,
    created: String,
    attached: bool,
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn health(&self) -> &str {
        "healthy"
    }

    async fn sessions(&self) -> async_graphql::Result<Vec<Session>> {
        let sessions = tmux::list_sessions()
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
        tmux::new_session(&name)
            .await
            .map_err(async_graphql::Error::new)
    }
}

pub fn build_schema() -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish()
}
