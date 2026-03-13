use tonic::{Request, Response, Status};

use super::tmux_gateway_proto::tmux_gateway_server::{TmuxGateway, TmuxGatewayServer};
use super::tmux_gateway_proto::{
    HealthCheckRequest, HealthCheckResponse, HelloRequest, HelloResponse,
};

pub struct TmuxGatewayServiceImpl;

#[tonic::async_trait]
impl TmuxGateway for TmuxGatewayServiceImpl {
    async fn check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        Ok(Response::new(HealthCheckResponse {
            status: "SERVING".to_string(),
        }))
    }

    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        let name = &request.get_ref().name;
        let name = if name.is_empty() { "World" } else { name };
        Ok(Response::new(HelloResponse {
            message: format!("Hello, {}!", name),
        }))
    }
}

pub fn grpc_server() -> TmuxGatewayServer<TmuxGatewayServiceImpl> {
    TmuxGatewayServer::new(TmuxGatewayServiceImpl)
}
