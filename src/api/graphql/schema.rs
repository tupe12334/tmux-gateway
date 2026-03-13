use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};

use crate::tmux;

pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

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

pub fn build_schema() -> AppSchema {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish()
}
