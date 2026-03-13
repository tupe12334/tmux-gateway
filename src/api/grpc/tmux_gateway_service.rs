use tonic::{Request, Response, Status};

use super::messages::{
    KillPaneRequest, KillPaneResponse, KillSessionRequest, KillSessionResponse, KillWindowRequest,
    KillWindowResponse, LsRequest, LsResponse, NewSessionRequest, NewSessionResponse, TmuxSession,
};
use super::server::{TmuxGateway, TmuxGatewayServer};
use crate::tmux::{self, TmuxCommands};

pub struct TmuxGatewayServiceImpl;

impl TmuxCommands for TmuxGatewayServiceImpl {
    async fn ls(&self) -> Result<Vec<tmux::TmuxSession>, String> {
        tmux::list_sessions().await
    }

    async fn new(&self, name: &str) -> Result<String, String> {
        tmux::new_session(name).await
    }

    async fn kill_session(&self, target: &str) -> Result<(), String> {
        tmux::kill_session(target).await
    }

    async fn kill_window(&self, target: &str) -> Result<(), String> {
        tmux::kill_window(target).await
    }

    async fn kill_pane(&self, target: &str) -> Result<(), String> {
        tmux::kill_pane(target).await
    }
}

#[tonic::async_trait]
impl TmuxGateway for TmuxGatewayServiceImpl {
    async fn ls(&self, _request: Request<LsRequest>) -> Result<Response<LsResponse>, Status> {
        let sessions = TmuxCommands::ls(self).await.map_err(Status::internal)?;

        let proto_sessions = sessions
            .into_iter()
            .map(|s| TmuxSession {
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
        let created_name = TmuxCommands::new(self, name)
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(NewSessionResponse { name: created_name }))
    }

    async fn kill_session(
        &self,
        request: Request<KillSessionRequest>,
    ) -> Result<Response<KillSessionResponse>, Status> {
        let target = &request.into_inner().target;
        TmuxCommands::kill_session(self, target)
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(KillSessionResponse {}))
    }

    async fn kill_window(
        &self,
        request: Request<KillWindowRequest>,
    ) -> Result<Response<KillWindowResponse>, Status> {
        let target = &request.into_inner().target;
        TmuxCommands::kill_window(self, target)
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(KillWindowResponse {}))
    }

    async fn kill_pane(
        &self,
        request: Request<KillPaneRequest>,
    ) -> Result<Response<KillPaneResponse>, Status> {
        let target = &request.into_inner().target;
        TmuxCommands::kill_pane(self, target)
            .await
            .map_err(Status::internal)?;
        Ok(Response::new(KillPaneResponse {}))
    }
}

pub fn grpc_server() -> TmuxGatewayServer<TmuxGatewayServiceImpl> {
    TmuxGatewayServer::new(TmuxGatewayServiceImpl)
}
