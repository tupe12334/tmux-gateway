use serde_json::{Map, Value, json};

/// Generates the AsyncAPI 3.0 specification for tmux-gateway's async messaging
/// interfaces: WebSocket pane streaming, gRPC server streaming, and GraphQL
/// subscriptions.
pub fn asyncapi_spec() -> Value {
    let mut spec = json!({
        "asyncapi": "3.0.0",
        "id": "urn:tmux-gateway",
        "info": {
            "title": "tmux-gateway",
            "version": "0.1.0",
            "description": "Async messaging interfaces for real-time tmux pane streaming",
            "license": { "name": "MIT" }
        }
    });

    let obj = spec.as_object_mut().expect("spec is an object");
    obj.insert("channels".into(), Value::Object(channels()));
    obj.insert("operations".into(), Value::Object(operations()));
    obj.insert("components".into(), Value::Object(components()));
    spec
}

fn channels() -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("wsPaneOutput".into(), json!({
        "address": "/ws/pane/{target}",
        "title": "WebSocket pane streaming",
        "description": "Real-time pane content pushed to clients over WebSocket. The server polls tmux at a configurable interval and sends a text frame only when the pane content changes.",
        "parameters": {
            "target": {
                "description": "Tmux pane target identifier (e.g. \"%0\", \"mysession:0.1\")"
            }
        },
        "messages": {
            "PaneContent": { "$ref": "#/components/messages/WsPaneContent" }
        },
        "bindings": {
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
        }
    }));
    m.insert("grpcStreamPaneOutput".into(), json!({
        "address": "tmux_gateway.TmuxGateway/StreamPaneOutput",
        "title": "gRPC server-streaming pane output",
        "description": "Server-side stream of pane output events via gRPC. The client sends a StreamPaneOutputRequest and receives a stream of StreamPaneOutputResponse messages, each containing pane content and a Unix timestamp. Only changed content is sent.",
        "messages": {
            "StreamPaneOutputResponse": { "$ref": "#/components/messages/GrpcStreamPaneOutputResponse" }
        }
    }));
    m.insert("graphqlPaneOutput".into(), json!({
        "address": "paneOutput",
        "title": "GraphQL pane output subscription",
        "description": "GraphQL subscription for pane output events over the /graphql/ws WebSocket endpoint. Yields PaneOutputEvent objects containing content and an RFC 3339 timestamp whenever the pane content changes.",
        "messages": {
            "PaneOutputEvent": { "$ref": "#/components/messages/GraphqlPaneOutputEvent" }
        }
    }));
    m
}

fn operations() -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("receiveWsPaneOutput".into(), json!({
        "action": "receive",
        "channel": { "$ref": "#/channels/wsPaneOutput" },
        "summary": "Receive real-time pane content via WebSocket",
        "description": "After upgrading the HTTP connection to WebSocket at /ws/pane/{target}, the server polls the target pane at the requested interval and pushes text frames containing the full pane content whenever it changes. The client may close the connection at any time."
    }));
    m.insert("receiveGrpcStreamPaneOutput".into(), json!({
        "action": "receive",
        "channel": { "$ref": "#/channels/grpcStreamPaneOutput" },
        "summary": "Receive pane output via gRPC server streaming",
        "description": "Initiates a server-streaming RPC. The client sends a StreamPaneOutputRequest with a target pane identifier and polling interval, then receives a stream of StreamPaneOutputResponse messages with content and timestamps."
    }));
    m.insert("receiveGraphqlPaneOutput".into(), json!({
        "action": "receive",
        "channel": { "$ref": "#/channels/graphqlPaneOutput" },
        "summary": "Subscribe to pane output via GraphQL subscription",
        "description": "Subscribes to the paneOutput field with a target pane and optional interval. Events are delivered over the /graphql/ws WebSocket transport as PaneOutputEvent objects."
    }));
    m
}

fn components() -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("messages".into(), json!({
        "WsPaneContent": {
            "name": "WsPaneContent",
            "title": "WebSocket pane content frame",
            "description": "Plain-text WebSocket message containing the full captured pane content",
            "contentType": "text/plain",
            "payload": { "$ref": "#/components/schemas/PaneContentText" }
        },
        "GrpcStreamPaneOutputResponse": {
            "name": "GrpcStreamPaneOutputResponse",
            "title": "gRPC StreamPaneOutputResponse",
            "description": "Protobuf-encoded streaming response with pane content and timestamp",
            "contentType": "application/protobuf",
            "payload": { "$ref": "#/components/schemas/StreamPaneOutputResponse" }
        },
        "GraphqlPaneOutputEvent": {
            "name": "GraphqlPaneOutputEvent",
            "title": "GraphQL PaneOutputEvent",
            "description": "GraphQL subscription event with pane content and RFC 3339 timestamp",
            "contentType": "application/json",
            "payload": { "$ref": "#/components/schemas/PaneOutputEvent" }
        }
    }));
    m.insert(
        "schemas".into(),
        json!({
            "PaneContentText": {
                "type": "string",
                "description": "Full captured content of a tmux pane"
            },
            "StreamPaneOutputRequest": {
                "type": "object",
                "description": "gRPC request to start streaming pane output",
                "required": ["target"],
                "properties": {
                    "target": {
                        "type": "string",
                        "description": "Tmux pane target identifier"
                    },
                    "interval_ms": {
                        "type": "integer",
                        "format": "uint32",
                        "description": "Polling interval in milliseconds (minimum 100)",
                        "default": 0
                    }
                }
            },
            "StreamPaneOutputResponse": {
                "type": "object",
                "description": "gRPC streaming response with pane content",
                "required": ["content", "timestamp"],
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Full captured pane content"
                    },
                    "timestamp": {
                        "type": "integer",
                        "format": "int64",
                        "description": "Unix timestamp (seconds since epoch)"
                    }
                }
            },
            "PaneOutputEvent": {
                "type": "object",
                "description": "GraphQL subscription event for pane output",
                "required": ["content", "timestamp"],
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Full captured pane content"
                    },
                    "timestamp": {
                        "type": "string",
                        "format": "date-time",
                        "description": "RFC 3339 timestamp of when the content was captured"
                    }
                }
            }
        }),
    );
    m
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
