pub mod messages;
mod schema;
pub mod server;
mod tmux_gateway_service;

pub use messages::*;
pub use server::{TmuxGateway, TmuxGatewayServer};
pub use tmux_gateway_service::{TmuxGatewayServiceImpl, grpc_server};

pub type TmuxGatewayServerConcrete =
    TmuxGatewayServer<tmux_gateway_service::TmuxGatewayServiceImpl>;

pub fn proto_content() -> String {
    format!(
        "syntax = \"proto3\";\n\npackage {};\n\n{}\n{}",
        server::package_name(),
        server::service_proto(),
        messages::messages_proto(),
    )
}

pub fn file_descriptor_set() -> prost_types::FileDescriptorSet {
    schema::compile_proto(&proto_content())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proto_content_is_valid() {
        let proto = proto_content();
        assert!(proto.contains("syntax = \"proto3\";"));
        assert!(proto.contains("package tmux_gateway;"));
        assert!(proto.contains("service TmuxGateway {"));
        assert!(proto.contains("rpc Ls(LsRequest) returns (LsResponse);"));
        assert!(proto.contains("rpc KillPane(KillPaneRequest) returns (KillPaneResponse);"));
        assert!(proto.contains("message LsRequest {}"));
        assert!(proto.contains("repeated TmuxSession sessions = 1;"));
        assert!(proto.contains("message TmuxSession {"));
        assert!(proto.contains("string name = 1;"));
        assert!(proto.contains("uint32 windows = 2;"));
        assert!(proto.contains("bool attached = 4;"));
        // New operations
        assert!(
            proto.contains("rpc ListWindows(ListWindowsRequest) returns (ListWindowsResponse);")
        );
        assert!(proto.contains("rpc ListPanes(ListPanesRequest) returns (ListPanesResponse);"));
        assert!(proto.contains("rpc SendKeys(SendKeysRequest) returns (SendKeysResponse);"));
        assert!(
            proto.contains(
                "rpc RenameSession(RenameSessionRequest) returns (RenameSessionResponse);"
            )
        );
        assert!(
            proto.contains("rpc RenameWindow(RenameWindowRequest) returns (RenameWindowResponse);")
        );
        assert!(proto.contains("rpc NewWindow(NewWindowRequest) returns (NewWindowResponse);"));
        assert!(
            proto.contains("rpc SplitWindow(SplitWindowRequest) returns (SplitWindowResponse);")
        );
        assert!(
            proto.contains("rpc CapturePane(CapturePaneRequest) returns (CapturePaneResponse);")
        );
        assert!(proto.contains("message TmuxWindow {"));
        assert!(proto.contains("message TmuxPaneMsg {"));
        assert!(proto.contains("repeated string keys = 2;"));
    }

    #[test]
    fn proto_compiles_with_protox() {
        // Verifies the generated proto is valid protobuf
        let _ = file_descriptor_set();
    }

    #[test]
    fn proto_has_all_rpcs() {
        let proto = proto_content();
        let expected_rpcs = [
            "rpc Ls(LsRequest) returns (LsResponse);",
            "rpc NewSession(NewSessionRequest) returns (NewSessionResponse);",
            "rpc KillSession(KillSessionRequest) returns (KillSessionResponse);",
            "rpc KillWindow(KillWindowRequest) returns (KillWindowResponse);",
            "rpc KillPane(KillPaneRequest) returns (KillPaneResponse);",
            "rpc ListWindows(ListWindowsRequest) returns (ListWindowsResponse);",
            "rpc ListPanes(ListPanesRequest) returns (ListPanesResponse);",
            "rpc SendKeys(SendKeysRequest) returns (SendKeysResponse);",
            "rpc RenameSession(RenameSessionRequest) returns (RenameSessionResponse);",
            "rpc RenameWindow(RenameWindowRequest) returns (RenameWindowResponse);",
            "rpc NewWindow(NewWindowRequest) returns (NewWindowResponse);",
            "rpc SplitWindow(SplitWindowRequest) returns (SplitWindowResponse);",
            "rpc CapturePane(CapturePaneRequest) returns (CapturePaneResponse);",
        ];
        for rpc in &expected_rpcs {
            assert!(proto.contains(rpc), "missing RPC: {rpc}");
        }
    }

    #[test]
    fn proto_has_all_messages() {
        let proto = proto_content();
        let expected_messages = [
            "message LsRequest",
            "message LsResponse",
            "message TmuxSession",
            "message NewSessionRequest",
            "message NewSessionResponse",
            "message KillSessionRequest",
            "message KillSessionResponse",
            "message KillWindowRequest",
            "message KillWindowResponse",
            "message KillPaneRequest",
            "message KillPaneResponse",
            "message ListWindowsRequest",
            "message ListWindowsResponse",
            "message TmuxWindow",
            "message ListPanesRequest",
            "message ListPanesResponse",
            "message TmuxPaneMsg",
            "message SendKeysRequest",
            "message SendKeysResponse",
            "message RenameSessionRequest",
            "message RenameSessionResponse",
            "message RenameWindowRequest",
            "message RenameWindowResponse",
            "message NewWindowRequest",
            "message NewWindowResponse",
            "message SplitWindowRequest",
            "message SplitWindowResponse",
            "message CapturePaneRequest",
            "message CapturePaneResponse",
        ];
        for msg in &expected_messages {
            assert!(proto.contains(msg), "missing message: {msg}");
        }
    }

    #[test]
    fn message_struct_construction() {
        let req = LsRequest {};
        let _ = req;

        let session = TmuxSession {
            name: "test".to_string(),
            windows: 3,
            created: "now".to_string(),
            attached: true,
        };
        assert_eq!(session.name, "test");
        assert_eq!(session.windows, 3);
        assert!(session.attached);

        let resp = LsResponse {
            sessions: vec![session],
        };
        assert_eq!(resp.sessions.len(), 1);

        let new_req = NewSessionRequest {
            name: "s1".to_string(),
        };
        assert_eq!(new_req.name, "s1");

        let kill_req = KillSessionRequest {
            target: "s1".to_string(),
        };
        assert_eq!(kill_req.target, "s1");
    }

    #[test]
    fn file_descriptor_set_has_service() {
        let fds = file_descriptor_set();
        let bytes = prost::Message::encode_to_vec(&fds);
        assert!(!bytes.is_empty());
        assert!(!fds.file.is_empty());
    }
}
