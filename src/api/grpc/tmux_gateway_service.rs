use tonic::{Request, Response, Status};

use super::messages::{
    CapturePaneRequest, CapturePaneResponse, KillPaneRequest, KillPaneResponse, KillSessionRequest,
    KillSessionResponse, KillWindowRequest, KillWindowResponse, ListPanesRequest,
    ListPanesResponse, ListWindowsRequest, ListWindowsResponse, LsRequest, LsResponse,
    NewSessionRequest, NewSessionResponse, NewWindowRequest, NewWindowResponse,
    RenameSessionRequest, RenameSessionResponse, RenameWindowRequest, RenameWindowResponse,
    SendKeysRequest, SendKeysResponse, SplitWindowRequest, SplitWindowResponse, TmuxPaneMsg,
    TmuxSession, TmuxWindow,
};
use super::server::{TmuxGateway, TmuxGatewayServer};
use crate::tmux::{self, GrpcCode, TmuxCommands, TmuxError};

pub struct TmuxGatewayServiceImpl;

impl TmuxCommands for TmuxGatewayServiceImpl {
    async fn ls(&self) -> Result<Vec<tmux::TmuxSession>, TmuxError> {
        tmux::list_sessions().await
    }

    async fn create_session(&self, name: &str) -> Result<String, TmuxError> {
        tmux::new_session(name).await
    }

    async fn kill_session(&self, target: &str) -> Result<(), TmuxError> {
        tmux::kill_session(target).await
    }

    async fn kill_window(&self, target: &str) -> Result<(), TmuxError> {
        tmux::kill_window(target).await
    }

    async fn kill_pane(&self, target: &str) -> Result<(), TmuxError> {
        tmux::kill_pane(target).await
    }

    async fn list_windows(&self, session: &str) -> Result<Vec<tmux::TmuxWindow>, TmuxError> {
        tmux::list_windows(session).await
    }

    async fn list_panes(&self, target: &str) -> Result<Vec<tmux::TmuxPane>, TmuxError> {
        tmux::list_panes(target).await
    }

    async fn send_keys(&self, target: &str, keys: &[String]) -> Result<(), TmuxError> {
        tmux::send_keys(target, keys).await
    }

    async fn rename_session(&self, target: &str, new_name: &str) -> Result<(), TmuxError> {
        tmux::rename_session(target, new_name).await
    }

    async fn rename_window(&self, target: &str, new_name: &str) -> Result<(), TmuxError> {
        tmux::rename_window(target, new_name).await
    }

    async fn new_window(&self, session: &str, name: &str) -> Result<String, TmuxError> {
        tmux::new_window(session, name).await
    }

    async fn split_window(&self, target: &str, horizontal: bool) -> Result<(), TmuxError> {
        tmux::split_window(target, horizontal).await
    }

    async fn capture_pane(&self, target: &str) -> Result<String, TmuxError> {
        tmux::capture_pane(target).await
    }
}

fn tmux_err_to_status(e: TmuxError) -> Status {
    let msg = e.to_string();
    match e.grpc_code() {
        GrpcCode::NotFound => Status::not_found(msg),
        GrpcCode::InvalidArgument => Status::invalid_argument(msg),
        GrpcCode::Internal => Status::internal(msg),
    }
}

#[tonic::async_trait]
impl TmuxGateway for TmuxGatewayServiceImpl {
    async fn ls(&self, _request: Request<LsRequest>) -> Result<Response<LsResponse>, Status> {
        let sessions = TmuxCommands::ls(self).await.map_err(tmux_err_to_status)?;

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
        let created_name = TmuxCommands::create_session(self, name)
            .await
            .map_err(tmux_err_to_status)?;
        Ok(Response::new(NewSessionResponse { name: created_name }))
    }

    async fn kill_session(
        &self,
        request: Request<KillSessionRequest>,
    ) -> Result<Response<KillSessionResponse>, Status> {
        let target = &request.into_inner().target;
        TmuxCommands::kill_session(self, target)
            .await
            .map_err(tmux_err_to_status)?;
        Ok(Response::new(KillSessionResponse {}))
    }

    async fn kill_window(
        &self,
        request: Request<KillWindowRequest>,
    ) -> Result<Response<KillWindowResponse>, Status> {
        let target = &request.into_inner().target;
        TmuxCommands::kill_window(self, target)
            .await
            .map_err(tmux_err_to_status)?;
        Ok(Response::new(KillWindowResponse {}))
    }

    async fn kill_pane(
        &self,
        request: Request<KillPaneRequest>,
    ) -> Result<Response<KillPaneResponse>, Status> {
        let target = &request.into_inner().target;
        TmuxCommands::kill_pane(self, target)
            .await
            .map_err(tmux_err_to_status)?;
        Ok(Response::new(KillPaneResponse {}))
    }

    async fn list_windows(
        &self,
        request: Request<ListWindowsRequest>,
    ) -> Result<Response<ListWindowsResponse>, Status> {
        let session = &request.into_inner().session;
        let windows = TmuxCommands::list_windows(self, session)
            .await
            .map_err(tmux_err_to_status)?;

        let proto_windows = windows
            .into_iter()
            .map(|w| TmuxWindow {
                index: w.index,
                name: w.name,
                panes: w.panes,
                active: w.active,
            })
            .collect();

        Ok(Response::new(ListWindowsResponse {
            windows: proto_windows,
        }))
    }

    async fn list_panes(
        &self,
        request: Request<ListPanesRequest>,
    ) -> Result<Response<ListPanesResponse>, Status> {
        let target = &request.into_inner().target;
        let panes = TmuxCommands::list_panes(self, target)
            .await
            .map_err(tmux_err_to_status)?;

        let proto_panes = panes
            .into_iter()
            .map(|p| TmuxPaneMsg {
                id: p.id,
                width: p.width,
                height: p.height,
                active: p.active,
            })
            .collect();

        Ok(Response::new(ListPanesResponse { panes: proto_panes }))
    }

    async fn send_keys(
        &self,
        request: Request<SendKeysRequest>,
    ) -> Result<Response<SendKeysResponse>, Status> {
        let inner = request.into_inner();
        TmuxCommands::send_keys(self, &inner.target, &inner.keys)
            .await
            .map_err(tmux_err_to_status)?;
        Ok(Response::new(SendKeysResponse {}))
    }

    async fn rename_session(
        &self,
        request: Request<RenameSessionRequest>,
    ) -> Result<Response<RenameSessionResponse>, Status> {
        let inner = request.into_inner();
        TmuxCommands::rename_session(self, &inner.target, &inner.new_name)
            .await
            .map_err(tmux_err_to_status)?;
        Ok(Response::new(RenameSessionResponse {}))
    }

    async fn rename_window(
        &self,
        request: Request<RenameWindowRequest>,
    ) -> Result<Response<RenameWindowResponse>, Status> {
        let inner = request.into_inner();
        TmuxCommands::rename_window(self, &inner.target, &inner.new_name)
            .await
            .map_err(tmux_err_to_status)?;
        Ok(Response::new(RenameWindowResponse {}))
    }

    async fn new_window(
        &self,
        request: Request<NewWindowRequest>,
    ) -> Result<Response<NewWindowResponse>, Status> {
        let inner = request.into_inner();
        let name = TmuxCommands::new_window(self, &inner.session, &inner.name)
            .await
            .map_err(tmux_err_to_status)?;
        Ok(Response::new(NewWindowResponse { name }))
    }

    async fn split_window(
        &self,
        request: Request<SplitWindowRequest>,
    ) -> Result<Response<SplitWindowResponse>, Status> {
        let inner = request.into_inner();
        TmuxCommands::split_window(self, &inner.target, inner.horizontal)
            .await
            .map_err(tmux_err_to_status)?;
        Ok(Response::new(SplitWindowResponse {}))
    }

    async fn capture_pane(
        &self,
        request: Request<CapturePaneRequest>,
    ) -> Result<Response<CapturePaneResponse>, Status> {
        let target = &request.into_inner().target;
        let content = TmuxCommands::capture_pane(self, target)
            .await
            .map_err(tmux_err_to_status)?;
        Ok(Response::new(CapturePaneResponse { content }))
    }
}

pub fn grpc_server() -> TmuxGatewayServer<TmuxGatewayServiceImpl> {
    TmuxGatewayServer::new(TmuxGatewayServiceImpl)
}
