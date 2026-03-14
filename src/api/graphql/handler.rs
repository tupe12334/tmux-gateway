use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::Router;
use axum::extract::Extension;
use axum::response::{Html, IntoResponse};
use axum::routing::get;

use super::schema::{AppSchema, build_schema};

async fn graphql_handler(schema: Extension<AppSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphiql() -> impl IntoResponse {
    Html(
        GraphiQLSource::build()
            .endpoint("/graphql")
            .subscription_endpoint("/graphql/ws")
            .finish(),
    )
}

pub fn router() -> Router {
    let schema = build_schema();
    Router::new()
        .route("/graphql", get(graphiql).post(graphql_handler))
        .route_service("/graphql/ws", GraphQLSubscription::new(schema.clone()))
        .layer(Extension(schema))
}
