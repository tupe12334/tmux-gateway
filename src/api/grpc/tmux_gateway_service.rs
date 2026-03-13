use tonic::{Request, Response, Status};

use super::tmux_gateway_proto::tmux_gateway_server::{TmuxGateway, TmuxGatewayServer};
use super::tmux_gateway_proto::{LsRequest, LsResponse, NewSessionRequest, NewSessionResponse};
use crate::tmux::{self, TmuxCommands};

pub struct TmuxGatewayServiceImpl;

impl TmuxCommands for TmuxGatewayServiceImpl {
    async fn ls(&self) -> Result<Vec<tmux::TmuxSession>, String> {
        tmux::list_sessions().await
    }

    async fn new_session(&self, name: &str) -> Result<String, String> {
        tmux::new_session(name).await
    }
}

#[tonic::async_trait]
impl TmuxGateway for TmuxGatewayServiceImpl {
    async fn ls(&self, _request: Request<LsRequest>) -> Result<Response<LsResponse>, Status> {
        let sessions = TmuxCommands::ls(self).await.map_err(Status::internal)?;

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

    async fn new_session(
        &self,
        request: Request<NewSessionRequest>,
    ) -> Result<Response<NewSessionResponse>, Status> {
        let name = &request.into_inner().name;
        let created_name = TmuxCommands::new_session(self, name)
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(NewSessionResponse { name: created_name }))
    }
}

pub fn grpc_server() -> TmuxGatewayServer<TmuxGatewayServiceImpl> {
    TmuxGatewayServer::new(TmuxGatewayServiceImpl)
}
