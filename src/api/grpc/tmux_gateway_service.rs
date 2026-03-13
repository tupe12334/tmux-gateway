use tonic::{Request, Response, Status};

use super::tmux_gateway_proto::tmux_gateway_server::{TmuxGateway, TmuxGatewayServer};
use super::tmux_gateway_proto::{LsRequest, LsResponse};
use crate::tmux;

pub struct TmuxGatewayServiceImpl;

#[tonic::async_trait]
impl TmuxGateway for TmuxGatewayServiceImpl {
    async fn ls(&self, _request: Request<LsRequest>) -> Result<Response<LsResponse>, Status> {
        let sessions = tmux::list_sessions().await.map_err(Status::internal)?;

        let proto_sessions = sessions
            .into_iter()
            .map(|s| super::tmux_gateway_proto::TmuxSession {
                name: s.name,
                windows: s.windows,
                created: s.created,
                attached: s.attached,
            })
            .collect();

        Ok(Response::new(LsResponse {
            sessions: proto_sessions,
        }))
    }
}

pub fn grpc_server() -> TmuxGatewayServer<TmuxGatewayServiceImpl> {
    TmuxGatewayServer::new(TmuxGatewayServiceImpl)
}
