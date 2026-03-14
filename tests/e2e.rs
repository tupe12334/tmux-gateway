mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tmux_gateway::api::{graphql, grpc, rest};
use tonic::Request as GrpcRequest;
use tower::ServiceExt;

use grpc::{KillSessionRequest, LsRequest, TmuxGateway, TmuxGatewayServiceImpl};

async fn body_json(body: Body) -> serde_json::Value {
    let bytes = body.collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn json_post(uri: &str, json: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(json.to_string()))
        .unwrap()
}

#[tokio::test]
async fn rest_session_lifecycle() {
    let session = common::TestSession::new();
    let app = rest::router();

    // Create
    let resp = app
        .clone()
        .oneshot(json_post(
            "/new",
            &format!(r#"{{"name":"{}"}}"#, session.name),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // List — session should appear
    let resp = app
        .clone()
        .oneshot(Request::builder().uri("/ls").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let sessions = body_json(resp.into_body()).await;
    assert!(
        sessions
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["name"] == session.name)
    );

    // Kill
    let resp = app
        .clone()
        .oneshot(json_post(
            "/kill-session",
            &format!(r#"{{"target":"{}"}}"#, session.name),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // List — session should be gone
    let resp = app
        .oneshot(Request::builder().uri("/ls").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let sessions = body_json(resp.into_body()).await;
    assert!(
        !sessions
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["name"] == session.name)
    );
}

#[tokio::test]
async fn cross_protocol_create_rest_verify_graphql_kill_grpc() {
    let session = common::TestSession::new();

    // Create via REST
    let app = rest::router();
    let resp = app
        .oneshot(json_post(
            "/new",
            &format!(r#"{{"name":"{}"}}"#, session.name),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Verify via GraphQL
    let schema = graphql::build_schema();
    let result = schema.execute("{ ls { name } }").await;
    assert!(result.errors.is_empty());
    let v = serde_json::to_value(&result.data).unwrap();
    assert!(
        v["ls"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["name"] == session.name)
    );

    // Kill via gRPC
    let service = TmuxGatewayServiceImpl;
    service
        .kill_session(GrpcRequest::new(KillSessionRequest {
            target: session.name.clone(),
        }))
        .await
        .unwrap();

    // Verify gone via REST
    let app = rest::router();
    let resp = app
        .oneshot(Request::builder().uri("/ls").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let sessions = body_json(resp.into_body()).await;
    assert!(
        !sessions
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["name"] == session.name)
    );
}

#[tokio::test]
async fn cross_protocol_create_graphql_verify_grpc_kill_rest() {
    let session = common::TestSession::new();

    // Create via GraphQL
    let schema = graphql::build_schema();
    let result = schema
        .execute(&format!(
            r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
            session.name
        ))
        .await;
    assert!(result.errors.is_empty());

    // Verify via gRPC
    let service = TmuxGatewayServiceImpl;
    let resp = service.ls(GrpcRequest::new(LsRequest {})).await.unwrap();
    assert!(
        resp.into_inner()
            .sessions
            .iter()
            .any(|s| s.name == session.name)
    );

    // Kill via REST
    let app = rest::router();
    let resp = app
        .oneshot(json_post(
            "/kill-session",
            &format!(r#"{{"target":"{}"}}"#, session.name),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn concurrent_session_creation() {
    let sessions: Vec<common::TestSession> = (0..5).map(|_| common::TestSession::new()).collect();
    let app = rest::router();

    let handles: Vec<_> = sessions
        .iter()
        .map(|session| {
            let app = app.clone();
            let name = session.name.clone();
            tokio::spawn(async move {
                app.oneshot(json_post("/new", &format!(r#"{{"name":"{}"}}"#, name)))
                    .await
                    .unwrap()
            })
        })
        .collect();

    for handle in handles {
        let resp = handle.await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    // Verify all exist
    let app = rest::router();
    let resp = app
        .oneshot(Request::builder().uri("/ls").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let body = body_json(resp.into_body()).await;
    let session_names: Vec<&str> = body
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|s| s["name"].as_str())
        .collect();
    for session in &sessions {
        assert!(
            session_names.contains(&session.name.as_str()),
            "session {} not found",
            session.name
        );
    }
}

#[tokio::test]
async fn all_apis_return_consistent_session_list() {
    let session = common::TestSession::new();

    // Create a session
    let schema = graphql::build_schema();
    let result = schema
        .execute(&format!(
            r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
            session.name
        ))
        .await;
    assert!(result.errors.is_empty());

    // Check REST
    let app = rest::router();
    let resp = app
        .oneshot(Request::builder().uri("/ls").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let rest_sessions = body_json(resp.into_body()).await;
    assert!(
        rest_sessions
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["name"] == session.name),
        "not in REST"
    );

    // Check GraphQL
    let result = schema.execute("{ ls { name } }").await;
    let v = serde_json::to_value(&result.data).unwrap();
    assert!(
        v["ls"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["name"] == session.name),
        "not in GraphQL"
    );

    // Check gRPC
    let service = TmuxGatewayServiceImpl;
    let resp = service.ls(GrpcRequest::new(LsRequest {})).await.unwrap();
    assert!(
        resp.into_inner()
            .sessions
            .iter()
            .any(|s| s.name == session.name),
        "not in gRPC"
    );
}
