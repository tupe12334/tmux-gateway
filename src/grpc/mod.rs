mod tmux_gateway_service;

pub mod tmux_gateway_proto {
    tonic::include_proto!("tmux_gateway");
}

pub const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("tmux_gateway_descriptor");

pub use tmux_gateway_service::grpc_server;
