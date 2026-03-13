pub mod messages;
mod schema;
pub mod server;
mod tmux_gateway_service;

use schema::ProtoSchema;

pub use messages::*;
pub use server::{TmuxGateway, TmuxGatewayServer};
pub use tmux_gateway_service::grpc_server;

pub type TmuxGatewayServerConcrete =
    TmuxGatewayServer<tmux_gateway_service::TmuxGatewayServiceImpl>;

fn proto_schema() -> ProtoSchema {
    ProtoSchema::new("tmux_gateway")
        .message("LsRequest", |_| {})
        .message("LsResponse", |m| {
            m.repeated_message("sessions", "TmuxSession", 1);
        })
        .message("TmuxSession", |m| {
            m.string("name", 1);
            m.uint32("windows", 2);
            m.string("created", 3);
            m.bool("attached", 4);
        })
        .message("NewSessionRequest", |m| {
            m.string("name", 1);
        })
        .message("NewSessionResponse", |m| {
            m.string("name", 1);
        })
        .service("TmuxGateway", |s| {
            s.unary("Ls", "LsRequest", "LsResponse");
            s.unary("NewSession", "NewSessionRequest", "NewSessionResponse");
        })
}

pub fn proto_content() -> String {
    proto_schema().proto_string()
}

pub fn file_descriptor_set() -> prost_types::FileDescriptorSet {
    proto_schema().file_descriptor_set()
}
