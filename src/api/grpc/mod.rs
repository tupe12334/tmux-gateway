pub mod messages;
mod schema;
pub mod server;
mod tmux_gateway_service;

pub use messages::*;
pub use server::{TmuxGateway, TmuxGatewayServer};
pub use tmux_gateway_service::grpc_server;

pub type TmuxGatewayServerConcrete =
    TmuxGatewayServer<tmux_gateway_service::TmuxGatewayServiceImpl>;

pub fn proto_content() -> &'static str {
    concat!(
        "syntax = \"proto3\";\n",
        "\n",
        "package tmux_gateway;\n",
        "\n",
        "service TmuxGateway {\n",
        "  rpc Ls(LsRequest) returns (LsResponse);\n",
        "  rpc NewSession(NewSessionRequest) returns (NewSessionResponse);\n",
        "  rpc KillSession(KillSessionRequest) returns (KillSessionResponse);\n",
        "  rpc KillWindow(KillWindowRequest) returns (KillWindowResponse);\n",
        "  rpc KillPane(KillPaneRequest) returns (KillPaneResponse);\n",
        "}\n",
        "\n",
        "message LsRequest {}\n",
        "\n",
        "message LsResponse {\n",
        "  repeated TmuxSession sessions = 1;\n",
        "}\n",
        "\n",
        "message TmuxSession {\n",
        "  string name = 1;\n",
        "  uint32 windows = 2;\n",
        "  string created = 3;\n",
        "  bool attached = 4;\n",
        "}\n",
        "\n",
        "message NewSessionRequest {\n",
        "  string name = 1;\n",
        "}\n",
        "\n",
        "message NewSessionResponse {\n",
        "  string name = 1;\n",
        "}\n",
        "\n",
        "message KillSessionRequest {\n",
        "  string target = 1;\n",
        "}\n",
        "\n",
        "message KillSessionResponse {}\n",
        "\n",
        "message KillWindowRequest {\n",
        "  string target = 1;\n",
        "}\n",
        "\n",
        "message KillWindowResponse {}\n",
        "\n",
        "message KillPaneRequest {\n",
        "  string target = 1;\n",
        "}\n",
        "\n",
        "message KillPaneResponse {}\n",
    )
}

pub fn file_descriptor_set() -> prost_types::FileDescriptorSet {
    schema::compile_proto(proto_content())
}
