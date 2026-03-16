use asyncapi_rust::{AsyncApi, ToAsyncApiMessage};
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::{Value, json};

// ── Message types (populate components/messages via derive) ──

/// WebSocket pane content (plain text frame).
#[derive(Serialize, JsonSchema, ToAsyncApiMessage)]
#[asyncapi(
    title = "WebSocket pane content frame",
    description = "Plain-text WebSocket message containing the full captured pane content",
    content_type = "text/plain"
)]
pub struct WsPaneContent(pub String);

/// gRPC StreamPaneOutputResponse.
#[derive(Serialize, JsonSchema, ToAsyncApiMessage)]
#[asyncapi(
    title = "gRPC StreamPaneOutputResponse",
    description = "Protobuf-encoded streaming response with pane content and timestamp",
    content_type = "application/protobuf"
)]
pub struct GrpcStreamPaneOutputResponse {
    /// Full captured pane content
    pub content: String,
    /// Unix timestamp (seconds since epoch)
    pub timestamp: i64,
}

/// GraphQL PaneOutputEvent.
#[derive(Serialize, JsonSchema, ToAsyncApiMessage)]
#[asyncapi(
    title = "GraphQL PaneOutputEvent",
    description = "GraphQL subscription event with pane content and RFC 3339 timestamp",
    content_type = "application/json"
)]
pub struct GraphqlPaneOutputEvent {
    /// Full captured pane content
    pub content: String,
    /// RFC 3339 timestamp of when the content was captured
    pub timestamp: String,
}

// ── Schema-only types (for components/schemas) ───────────────

/// gRPC request to start streaming pane output.
#[derive(Serialize, JsonSchema)]
pub struct StreamPaneOutputRequest {
    /// Tmux pane target identifier
    pub target: String,
    /// Polling interval in milliseconds (minimum 100)
    #[serde(default)]
    pub interval_ms: u32,
}

// ── AsyncAPI spec definition ─────────────────────────────────

#[allow(clippy::duplicated_attributes)]
#[derive(AsyncApi)]
#[asyncapi(
    title = "tmux-gateway",
    version = "0.1.0",
    description = "Async messaging interfaces for real-time tmux pane streaming"
)]
#[asyncapi_channel(
    name = "wsPaneOutput",
    address = "/ws/pane/{target}",
    parameter(
        name = "target",
        description = "Tmux pane target identifier (e.g. \"%0\", \"mysession:0.1\")"
    )
)]
#[asyncapi_channel(
    name = "grpcStreamPaneOutput",
    address = "tmux_gateway.TmuxGateway/StreamPaneOutput"
)]
#[asyncapi_channel(name = "graphqlPaneOutput", address = "paneOutput")]
#[asyncapi_operation(
    name = "receiveWsPaneOutput",
    action = "receive",
    channel = "wsPaneOutput"
)]
#[asyncapi_operation(
    name = "receiveGrpcStreamPaneOutput",
    action = "receive",
    channel = "grpcStreamPaneOutput"
)]
#[asyncapi_operation(
    name = "receiveGraphqlPaneOutput",
    action = "receive",
    channel = "graphqlPaneOutput"
)]
#[asyncapi_messages(WsPaneContent, GrpcStreamPaneOutputResponse, GraphqlPaneOutputEvent)]
struct TmuxGatewayAsyncApi;

/// Generates the AsyncAPI 3.0 specification for tmux-gateway's async messaging
/// interfaces: WebSocket pane streaming, gRPC server streaming, and GraphQL
/// subscriptions.
pub fn asyncapi_spec() -> Value {
    let spec = TmuxGatewayAsyncApi::asyncapi_spec();
    let mut value = serde_json::to_value(&spec).expect("spec serializes to JSON");
    enrich(&mut value);
    value
}

/// Enriches the derive-generated spec with fields the crate does not yet support.
fn enrich(spec: &mut Value) {
    let obj = spec.as_object_mut().expect("spec is an object");
    obj.insert("id".into(), json!("urn:tmux-gateway"));

    if let Some(info) = obj.get_mut("info").and_then(Value::as_object_mut) {
        info.insert("license".into(), json!({ "name": "MIT" }));
    }

    enrich_channels(obj);
    enrich_operations(obj);
    enrich_component_schemas(obj);
}

fn enrich_channels(obj: &mut serde_json::Map<String, Value>) {
    let channels = match obj.get_mut("channels").and_then(Value::as_object_mut) {
        Some(c) => c,
        None => return,
    };

    if let Some(ws) = channels
        .get_mut("wsPaneOutput")
        .and_then(Value::as_object_mut)
    {
        ws.insert("title".into(), json!("WebSocket pane streaming"));
        ws.insert(
            "description".into(),
            json!("Real-time pane content pushed to clients over WebSocket. The server polls tmux at a configurable interval and sends a text frame only when the pane content changes."),
        );
        ws.insert(
            "messages".into(),
            json!({ "PaneContent": { "$ref": "#/components/messages/WsPaneContent" } }),
        );
        ws.insert(
            "bindings".into(),
            json!({
                "ws": {
                    "query": {
                        "type": "object",
                        "properties": {
                            "interval_ms": {
                                "type": "integer",
                                "description": "Polling interval in milliseconds (minimum 100)",
                                "default": 500,
                                "minimum": 100
                            }
                        }
                    }
                }
            }),
        );
    }

    if let Some(grpc) = channels
        .get_mut("grpcStreamPaneOutput")
        .and_then(Value::as_object_mut)
    {
        grpc.insert("title".into(), json!("gRPC server-streaming pane output"));
        grpc.insert(
            "description".into(),
            json!("Server-side stream of pane output events via gRPC. The client sends a StreamPaneOutputRequest and receives a stream of StreamPaneOutputResponse messages, each containing pane content and a Unix timestamp. Only changed content is sent."),
        );
        grpc.insert(
            "messages".into(),
            json!({ "StreamPaneOutputResponse": { "$ref": "#/components/messages/GrpcStreamPaneOutputResponse" } }),
        );
    }

    if let Some(gql) = channels
        .get_mut("graphqlPaneOutput")
        .and_then(Value::as_object_mut)
    {
        gql.insert("title".into(), json!("GraphQL pane output subscription"));
        gql.insert(
            "description".into(),
            json!("GraphQL subscription for pane output events over the /graphql/ws WebSocket endpoint. Yields PaneOutputEvent objects containing content and an RFC 3339 timestamp whenever the pane content changes."),
        );
        gql.insert(
            "messages".into(),
            json!({ "PaneOutputEvent": { "$ref": "#/components/messages/GraphqlPaneOutputEvent" } }),
        );
    }
}

fn enrich_operations(obj: &mut serde_json::Map<String, Value>) {
    let ops = match obj.get_mut("operations").and_then(Value::as_object_mut) {
        Some(o) => o,
        None => return,
    };

    let enrichments = [
        (
            "receiveWsPaneOutput",
            "Receive real-time pane content via WebSocket",
            "After upgrading the HTTP connection to WebSocket at /ws/pane/{target}, the server polls the target pane at the requested interval and pushes text frames containing the full pane content whenever it changes. The client may close the connection at any time.",
        ),
        (
            "receiveGrpcStreamPaneOutput",
            "Receive pane output via gRPC server streaming",
            "Initiates a server-streaming RPC. The client sends a StreamPaneOutputRequest with a target pane identifier and polling interval, then receives a stream of StreamPaneOutputResponse messages with content and timestamps.",
        ),
        (
            "receiveGraphqlPaneOutput",
            "Subscribe to pane output via GraphQL subscription",
            "Subscribes to the paneOutput field with a target pane and optional interval. Events are delivered over the /graphql/ws WebSocket transport as PaneOutputEvent objects.",
        ),
    ];

    for (name, summary, description) in enrichments {
        if let Some(op) = ops.get_mut(name).and_then(Value::as_object_mut) {
            op.insert("summary".into(), json!(summary));
            op.insert("description".into(), json!(description));
        }
    }
}

fn enrich_component_schemas(obj: &mut serde_json::Map<String, Value>) {
    let components = obj.entry("components").or_insert_with(|| json!({}));
    let components = components.as_object_mut().expect("components is an object");

    let request_schema = serde_json::to_value(schemars::schema_for!(StreamPaneOutputRequest))
        .expect("schema serializes");
    let response_schema = serde_json::to_value(schemars::schema_for!(GrpcStreamPaneOutputResponse))
        .expect("schema serializes");
    let event_schema = serde_json::to_value(schemars::schema_for!(GraphqlPaneOutputEvent))
        .expect("schema serializes");

    components.insert(
        "schemas".into(),
        json!({
            "PaneContentText": {
                "type": "string",
                "description": "Full captured content of a tmux pane"
            },
            "StreamPaneOutputRequest": request_schema,
            "StreamPaneOutputResponse": response_schema,
            "PaneOutputEvent": event_schema
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_has_asyncapi_version() {
        let spec = asyncapi_spec();
        assert_eq!(spec["asyncapi"], "3.0.0");
    }

    #[test]
    fn spec_has_info() {
        let spec = asyncapi_spec();
        assert_eq!(spec["info"]["title"], "tmux-gateway");
        assert!(spec["info"]["version"].is_string());
        assert!(spec["info"]["license"]["name"].is_string());
    }

    #[test]
    fn spec_has_all_channels() {
        let spec = asyncapi_spec();
        let channels = spec["channels"].as_object().unwrap();
        assert!(
            channels.contains_key("wsPaneOutput"),
            "missing wsPaneOutput channel"
        );
        assert!(
            channels.contains_key("grpcStreamPaneOutput"),
            "missing grpcStreamPaneOutput channel"
        );
        assert!(
            channels.contains_key("graphqlPaneOutput"),
            "missing graphqlPaneOutput channel"
        );
    }

    #[test]
    fn ws_channel_has_address_and_params() {
        let spec = asyncapi_spec();
        let ch = &spec["channels"]["wsPaneOutput"];
        assert_eq!(ch["address"], "/ws/pane/{target}");
        assert!(ch["parameters"]["target"].is_object());
    }

    #[test]
    fn grpc_channel_has_address() {
        let spec = asyncapi_spec();
        let ch = &spec["channels"]["grpcStreamPaneOutput"];
        assert_eq!(ch["address"], "tmux_gateway.TmuxGateway/StreamPaneOutput");
    }

    #[test]
    fn graphql_channel_has_address() {
        let spec = asyncapi_spec();
        let ch = &spec["channels"]["graphqlPaneOutput"];
        assert_eq!(ch["address"], "paneOutput");
    }

    #[test]
    fn spec_has_all_operations() {
        let spec = asyncapi_spec();
        let ops = spec["operations"].as_object().unwrap();
        assert!(ops.contains_key("receiveWsPaneOutput"));
        assert!(ops.contains_key("receiveGrpcStreamPaneOutput"));
        assert!(ops.contains_key("receiveGraphqlPaneOutput"));
        for (_, op) in ops {
            assert_eq!(op["action"], "receive");
        }
    }

    #[test]
    fn spec_has_component_messages() {
        let spec = asyncapi_spec();
        let msgs = spec["components"]["messages"].as_object().unwrap();
        assert!(msgs.contains_key("WsPaneContent"));
        assert!(msgs.contains_key("GrpcStreamPaneOutputResponse"));
        assert!(msgs.contains_key("GraphqlPaneOutputEvent"));
    }

    #[test]
    fn spec_has_component_schemas() {
        let spec = asyncapi_spec();
        let schemas = spec["components"]["schemas"].as_object().unwrap();
        assert!(schemas.contains_key("PaneContentText"));
        assert!(schemas.contains_key("StreamPaneOutputRequest"));
        assert!(schemas.contains_key("StreamPaneOutputResponse"));
        assert!(schemas.contains_key("PaneOutputEvent"));
    }

    #[test]
    fn spec_serializes_to_valid_json() {
        let spec = asyncapi_spec();
        let json_str = serde_json::to_string_pretty(&spec).unwrap();
        let roundtrip: Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(spec, roundtrip);
    }
}
