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
    if !common::tmux_available() {
        return;
    }
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
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let schema = graphql::build_schema();
    let query = format!(
        r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
        name
    );
    let result = schema.execute(&query).await;
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    let v = to_json(&result.data);
    assert_eq!(v["createSession"]["name"], name);
    common::cleanup_session(&name);
}

#[tokio::test]
async fn kill_session_mutation() {
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let schema = graphql::build_schema();
    let query = format!(
        r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
        name
    );
    schema.execute(&query).await;
    let query = format!(r#"mutation {{ killSession(target: "{}") }}"#, name);
    let result = schema.execute(&query).await;
    assert!(result.errors.is_empty());
    let v = to_json(&result.data);
    assert_eq!(v["killSession"], true);
}

#[tokio::test]
async fn kill_window_mutation() {
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let schema = graphql::build_schema();
    let query = format!(
        r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
        name
    );
    schema.execute(&query).await;
    let query = format!(r#"mutation {{ killWindow(target: "{}:0") }}"#, name);
    let result = schema.execute(&query).await;
    assert!(result.errors.is_empty());
    assert_eq!(to_json(&result.data)["killWindow"], true);
}

#[tokio::test]
async fn kill_pane_mutation() {
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let schema = graphql::build_schema();
    let query = format!(
        r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
        name
    );
    schema.execute(&query).await;
    let query = format!(r#"mutation {{ killPane(target: "{}:0.0") }}"#, name);
    let result = schema.execute(&query).await;
    assert!(result.errors.is_empty());
    assert_eq!(to_json(&result.data)["killPane"], true);
}

#[tokio::test]
async fn list_windows_query() {
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let schema = graphql::build_schema();
    schema
        .execute(&format!(
            r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
            name
        ))
        .await;
    let query = format!(
        r#"{{ listWindows(session: "{}") {{ index name panes active }} }}"#,
        name
    );
    let result = schema.execute(&query).await;
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    let v = to_json(&result.data);
    assert!(v["listWindows"].is_array());
    assert!(!v["listWindows"].as_array().unwrap().is_empty());
    common::cleanup_session(&name);
}

#[tokio::test]
async fn list_panes_query() {
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let schema = graphql::build_schema();
    schema
        .execute(&format!(
            r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
            name
        ))
        .await;
    let query = format!(
        r#"{{ listPanes(target: "{}:0") {{ id width height active currentPath currentCommand }} }}"#,
        name
    );
    let result = schema.execute(&query).await;
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    common::cleanup_session(&name);
}

#[tokio::test]
async fn send_keys_mutation() {
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let schema = graphql::build_schema();
    schema
        .execute(&format!(
            r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
            name
        ))
        .await;
    let query = format!(
        r#"mutation {{ sendKeys(target: "{}:0.0", keys: ["echo", "Enter"]) }}"#,
        name
    );
    let result = schema.execute(&query).await;
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert_eq!(to_json(&result.data)["sendKeys"], true);
    common::cleanup_session(&name);
}

#[tokio::test]
async fn rename_session_mutation() {
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let new_name = common::unique_session_name();
    let schema = graphql::build_schema();
    schema
        .execute(&format!(
            r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
            name
        ))
        .await;
    let query = format!(
        r#"mutation {{ renameSession(target: "{}", newName: "{}") }}"#,
        name, new_name
    );
    let result = schema.execute(&query).await;
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    common::cleanup_session(&new_name);
}

#[tokio::test]
async fn new_window_mutation() {
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let schema = graphql::build_schema();
    schema
        .execute(&format!(
            r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
            name
        ))
        .await;
    let query = format!(
        r#"mutation {{ newWindow(session: "{}", name: "mywin") {{ name }} }}"#,
        name
    );
    let result = schema.execute(&query).await;
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    common::cleanup_session(&name);
}

#[tokio::test]
async fn rename_window_mutation() {
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let schema = graphql::build_schema();
    schema
        .execute(&format!(
            r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
            name
        ))
        .await;
    let query = format!(
        r#"mutation {{ renameWindow(target: "{}:0", newName: "renamed") }}"#,
        name
    );
    let result = schema.execute(&query).await;
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    common::cleanup_session(&name);
}

#[tokio::test]
async fn split_window_mutation() {
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let schema = graphql::build_schema();
    schema
        .execute(&format!(
            r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
            name
        ))
        .await;
    let query = format!(
        r#"mutation {{ splitWindow(target: "{}:0.0", horizontal: false) {{ id }} }}"#,
        name
    );
    let result = schema.execute(&query).await;
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    common::cleanup_session(&name);
}

#[tokio::test]
async fn capture_pane_query() {
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let schema = graphql::build_schema();
    schema
        .execute(&format!(
            r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
            name
        ))
        .await;
    let query = format!(r#"{{ capturePane(target: "{}:0.0") }}"#, name);
    let result = schema.execute(&query).await;
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    common::cleanup_session(&name);
}

#[tokio::test]
async fn kill_session_nonexistent_returns_error() {
    if !common::tmux_available() {
        return;
    }
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
    if !common::tmux_available() {
        return;
    }
    let name = common::unique_session_name();
    let schema = graphql::build_schema();
    schema
        .execute(&format!(
            r#"mutation {{ createSession(name: "{}") {{ name }} }}"#,
            name
        ))
        .await;
    let result = schema.execute("{ ls { name } }").await;
    assert!(result.errors.is_empty());
    let v = to_json(&result.data);
    let sessions = v["ls"].as_array().unwrap();
    assert!(
        sessions.iter().any(|s| s["name"] == name),
        "session {} not found in ls",
        name
    );
    common::cleanup_session(&name);
}
