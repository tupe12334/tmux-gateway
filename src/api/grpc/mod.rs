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
    include_str!("tmux_gateway.proto")
}

pub fn file_descriptor_set() -> prost_types::FileDescriptorSet {
    schema::compile_proto(proto_content())
}
