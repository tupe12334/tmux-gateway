use super::tmux_gateway_proto::tmux_gateway_server::{TmuxGateway, TmuxGatewayServer};
use tonic::{Request, Response, Status};

pub struct TmuxGatewayServiceImpl;

#[tonic::async_trait]
impl TmuxGateway for TmuxGatewayServiceImpl {}

pub fn grpc_server() -> TmuxGatewayServer<TmuxGatewayServiceImpl> {
    TmuxGatewayServer::new(TmuxGatewayServiceImpl)
}
