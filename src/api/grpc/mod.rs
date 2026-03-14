pub mod messages;
mod schema;
pub mod server;
mod tmux_gateway_service;

pub use messages::*;
pub use server::{TmuxGateway, TmuxGatewayServer};
pub use tmux_gateway_service::grpc_server;

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
}
