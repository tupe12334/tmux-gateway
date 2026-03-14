mod common;

use tmux_gateway::api::graphql;

fn to_json(data: &async_graphql::Value) -> serde_json::Value {
    serde_json::to_value(data).unwrap()
}

#[tokio::test]
async fn health_query() {
    let schema = graphql::build_schema();
    let result = schema.execute("{ health }").await;
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    let v = to_json(&result.data);
    assert_eq!(v["health"], "healthy");
}

#[tokio::test]
async fn ls_returns_array() {
    common::require_tmux();
    let schema = graphql::build_schema();
    let result = schema
        .execute("{ ls { name windows created attached } }")
        .await;
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    let v = to_json(&result.data);
    assert!(v["ls"].is_array());
}

#[tokio::test]
async fn create_session_mutation() {
    let session = common::TestSession::new();
    let schema = graphql::build_schema();
    let query = format!(
        r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
        session.name
    );
    let result = schema.execute(&query).await;
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    let v = to_json(&result.data);
    assert_eq!(v["createSession"]["name"], session.name);
}

#[tokio::test]
async fn kill_session_mutation() {
    let session = common::TestSession::new();
    let schema = graphql::build_schema();
    let query = format!(
        r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
        session.name
    );
    schema.execute(&query).await;
    let query = format!(r#"mutation {{ killSession(target: "{}") }}"#, session.name);
    let result = schema.execute(&query).await;
    assert!(result.errors.is_empty());
    let v = to_json(&result.data);
    assert_eq!(v["killSession"], true);
}

#[tokio::test]
async fn kill_window_mutation() {
    let session = common::TestSession::new();
    let schema = graphql::build_schema();
    let query = format!(
        r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
        session.name
    );
    schema.execute(&query).await;
    let query = format!(r#"mutation {{ killWindow(target: "{}:0") }}"#, session.name);
    let result = schema.execute(&query).await;
    assert!(result.errors.is_empty());
    assert_eq!(to_json(&result.data)["killWindow"], true);
}

#[tokio::test]
async fn kill_pane_mutation() {
    let session = common::TestSession::new();
    let schema = graphql::build_schema();
    let query = format!(
        r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
        session.name
    );
    schema.execute(&query).await;
    let query = format!(r#"mutation {{ killPane(target: "{}:0.0") }}"#, session.name);
    let result = schema.execute(&query).await;
    assert!(result.errors.is_empty());
    assert_eq!(to_json(&result.data)["killPane"], true);
}

#[tokio::test]
async fn list_windows_query() {
    let session = common::TestSession::new();
    let schema = graphql::build_schema();
    schema
        .execute(&format!(
            r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
            session.name
        ))
        .await;
    let query = format!(
        r#"{{ listWindows(session: "{}") {{ index name panes active }} }}"#,
        session.name
    );
    let result = schema.execute(&query).await;
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    let v = to_json(&result.data);
    assert!(v["listWindows"].is_array());
    assert!(!v["listWindows"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn list_panes_query() {
    let session = common::TestSession::new();
    let schema = graphql::build_schema();
    schema
        .execute(&format!(
            r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
            session.name
        ))
        .await;
    let query = format!(
        r#"{{ listPanes(target: "{}:0") {{ id width height active currentPath currentCommand }} }}"#,
        session.name
    );
    let result = schema.execute(&query).await;
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
}

#[tokio::test]
async fn send_keys_mutation() {
    let session = common::TestSession::new();
    let schema = graphql::build_schema();
    schema
        .execute(&format!(
            r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
            session.name
        ))
        .await;
    let query = format!(
        r#"mutation {{ sendKeys(target: "{}:0.0", keys: ["echo", "Enter"]) }}"#,
        session.name
    );
    let result = schema.execute(&query).await;
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(to_json(&result.data)["sendKeys"], true);
}

#[tokio::test]
async fn rename_session_mutation() {
    let session = common::TestSession::new();
    let new_session = common::TestSession::new();
    let schema = graphql::build_schema();
    schema
        .execute(&format!(
            r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
            session.name
        ))
        .await;
    let query = format!(
        r#"mutation {{ renameSession(target: "{}", newName: "{}") }}"#,
        session.name, new_session.name
    );
    let result = schema.execute(&query).await;
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
}

#[tokio::test]
async fn new_window_mutation() {
    let session = common::TestSession::new();
    let schema = graphql::build_schema();
    schema
        .execute(&format!(
            r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
            session.name
        ))
        .await;
    let query = format!(
        r#"mutation {{ newWindow(session: "{}", name: "mywin") {{ name }} }}"#,
        session.name
    );
    let result = schema.execute(&query).await;
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
}

#[tokio::test]
async fn rename_window_mutation() {
    let session = common::TestSession::new();
    let schema = graphql::build_schema();
    schema
        .execute(&format!(
            r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
            session.name
        ))
        .await;
    let query = format!(
        r#"mutation {{ renameWindow(target: "{}:0", newName: "renamed") }}"#,
        session.name
    );
    let result = schema.execute(&query).await;
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
}

#[tokio::test]
async fn split_window_mutation() {
    let session = common::TestSession::new();
    let schema = graphql::build_schema();
    schema
        .execute(&format!(
            r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
            session.name
        ))
        .await;
    let query = format!(
        r#"mutation {{ splitWindow(target: "{}:0.0", horizontal: false) {{ id }} }}"#,
        session.name
    );
    let result = schema.execute(&query).await;
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
}

#[tokio::test]
async fn capture_pane_query() {
    let session = common::TestSession::new();
    let schema = graphql::build_schema();
    schema
        .execute(&format!(
            r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
            session.name
        ))
        .await;
    let query = format!(r#"{{ capturePane(target: "{}:0.0") }}"#, session.name);
    let result = schema.execute(&query).await;
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
}

#[tokio::test]
async fn kill_session_nonexistent_returns_error() {
    common::require_tmux();
    let schema = graphql::build_schema();
    let result = schema
        .execute(r#"mutation { killSession(target: "nonexistent_xyz_99999") }"#)
        .await;
    assert!(!result.errors.is_empty());
}

#[tokio::test]
async fn invalid_query_returns_error() {
    let schema = graphql::build_schema();
    let result = schema.execute("{ nonExistentField }").await;
    assert!(!result.errors.is_empty());
}

#[tokio::test]
async fn ls_includes_created_session() {
    let session = common::TestSession::new();
    let schema = graphql::build_schema();
    schema
        .execute(&format!(
            r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
            session.name
        ))
        .await;
    let result = schema.execute("{ ls { name } }").await;
    assert!(result.errors.is_empty());
    let v = to_json(&result.data);
    let sessions = v["ls"].as_array().unwrap();
    assert!(
        sessions.iter().any(|s| s["name"] == session.name),
        "session {} not found in ls",
        session.name
    );
}
