use axum::extract::Path;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};

#[derive(Serialize, ToSchema)]
struct HealthResponse {
    status: String,
}

#[derive(Serialize, ToSchema)]
struct HelloResponse {
    message: String,
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health check", body = HealthResponse)
    )
)]
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
    })
}

#[utoipa::path(
    get,
    path = "/hello/{name}",
    params(
        ("name" = String, Path, description = "Name to greet")
    ),
    responses(
        (status = 200, description = "Greeting message", body = HelloResponse)
    )
)]
async fn hello(Path(name): Path<String>) -> Json<HelloResponse> {
    Json(HelloResponse {
        message: format!("Hello, {}!", name),
    })
}

#[derive(OpenApi)]
#[openapi(
    paths(health, hello),
    components(schemas(HealthResponse, HelloResponse)),
    info(
        title = "tmux-gateway",
        version = "0.1.0",
        description = "REST API for interacting with local tmux sessions"
    )
)]
pub struct ApiDoc;

pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/hello/{name}", get(hello))
}
