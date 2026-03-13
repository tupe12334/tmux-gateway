mod tmux_gateway_service;

#[allow(clippy::match_single_binding)]
pub mod tmux_gateway_proto {
    tonic::include_proto!("tmux_gateway");
}

pub const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("tmux_gateway_descriptor");

include!(concat!(env!("OUT_DIR"), "/proto_content.rs"));

pub use tmux_gateway_service::grpc_server;

pub type TmuxGatewayServer = tmux_gateway_proto::tmux_gateway_server::TmuxGatewayServer<
    tmux_gateway_service::TmuxGatewayServiceImpl,
>;
