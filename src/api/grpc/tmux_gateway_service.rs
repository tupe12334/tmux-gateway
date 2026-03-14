use tonic::{Request, Response, Status};

use super::messages::{
    CapturePaneRequest, CapturePaneResponse, CapturePaneWithOptionsRequest,
    CapturePaneWithOptionsResponse, CreateSessionWithWindowsRequest,
    CreateSessionWithWindowsResponse, KillPaneRequest, KillPaneResponse, KillSessionRequest,
    KillSessionResponse, KillWindowRequest, KillWindowResponse, ListPanesRequest,
    ListPanesResponse, ListWindowsRequest, ListWindowsResponse, LsRequest, LsResponse,
    MoveWindowRequest, MoveWindowResponse, NewSessionRequest, NewSessionResponse, NewWindowRequest,
    NewWindowResponse, RenameSessionRequest, RenameSessionResponse, RenameWindowRequest,
    RenameWindowResponse, SelectPaneRequest, SelectPaneResponse, SelectWindowRequest,
    SelectWindowResponse, SendKeysRequest, SendKeysResponse, SplitWindowRequest,
    SplitWindowResponse, SwapPanesRequest, SwapPanesResponse, TmuxPaneMsg, TmuxSession, TmuxWindow,
};
use super::server::{TmuxGateway, TmuxGatewayServer};
use crate::tmux::{self, RealTmuxExecutor, TmuxCommands, TmuxError};

pub struct TmuxGatewayServiceImpl;

impl TmuxCommands for TmuxGatewayServiceImpl {
    async fn ls(&self) -> Result<Vec<tmux::TmuxSession>, TmuxError> {
        tmux::list_sessions(&RealTmuxExecutor).await
    }

    async fn create_session(&self, name: &str) -> Result<tmux::TmuxSession, TmuxError> {
        tmux::new_session(&RealTmuxExecutor, name).await
    }

    async fn kill_session(&self, target: &str) -> Result<(), TmuxError> {
        tmux::kill_session(&RealTmuxExecutor, target).await
    }

    async fn kill_window(&self, target: &str) -> Result<(), TmuxError> {
        tmux::kill_window(&RealTmuxExecutor, target).await
    }

    async fn kill_pane(&self, target: &str) -> Result<(), TmuxError> {
        tmux::kill_pane(&RealTmuxExecutor, target).await
    }

    async fn list_windows(&self, session: &str) -> Result<Vec<tmux::TmuxWindow>, TmuxError> {
        tmux::list_windows(&RealTmuxExecutor, session).await
    }

    async fn list_panes(&self, target: &str) -> Result<Vec<tmux::TmuxPane>, TmuxError> {
        tmux::list_panes(&RealTmuxExecutor, target).await
    }

    async fn send_keys(&self, target: &str, keys: &[String]) -> Result<(), TmuxError> {
        tmux::send_keys(&RealTmuxExecutor, target, keys).await
    }

    async fn rename_session(&self, target: &str, new_name: &str) -> Result<(), TmuxError> {
        tmux::rename_session(&RealTmuxExecutor, target, new_name).await
    }

    async fn rename_window(&self, target: &str, new_name: &str) -> Result<(), TmuxError> {
        tmux::rename_window(&RealTmuxExecutor, target, new_name).await
    }

    async fn new_window(&self, session: &str, name: &str) -> Result<tmux::TmuxWindow, TmuxError> {
        tmux::new_window(&RealTmuxExecutor, session, name).await
    }

    async fn split_window(
        &self,
        target: &str,
        horizontal: bool,
    ) -> Result<tmux::TmuxPane, TmuxError> {
        tmux::split_window(&RealTmuxExecutor, target, horizontal).await
    }

    async fn capture_pane(&self, target: &str) -> Result<String, TmuxError> {
        tmux::capture_pane(&RealTmuxExecutor, target).await
    }

    async fn capture_pane_with_options(
        &self,
        target: &str,
        opts: &tmux::CaptureOptions,
    ) -> Result<String, TmuxError> {
        tmux::capture_pane_with_options(&RealTmuxExecutor, target, opts).await
    }

    async fn create_session_with_windows(
        &self,
        name: &str,
        window_names: &[String],
    ) -> Result<tmux::TmuxSession, TmuxError> {
        tmux::create_session_with_windows(&RealTmuxExecutor, name, window_names).await
    }

    async fn swap_panes(&self, src: &str, dst: &str) -> Result<(), TmuxError> {
        tmux::swap_panes(&RealTmuxExecutor, src, dst).await
    }

    async fn move_window(&self, source: &str, destination_session: &str) -> Result<(), TmuxError> {
        tmux::move_window(&RealTmuxExecutor, source, destination_session).await
    }

    async fn select_window(&self, target: &str) -> Result<(), TmuxError> {
        tmux::select_window(&RealTmuxExecutor, target).await
    }

    async fn select_pane(&self, target: &str) -> Result<(), TmuxError> {
        tmux::select_pane(&RealTmuxExecutor, target).await
    }
}

fn tmux_err_to_status(e: TmuxError) -> Status {
    let msg = e.to_string();
    match e {
        TmuxError::SessionNotFound(_)
        | TmuxError::WindowNotFound(_)
        | TmuxError::PaneNotFound(_) => Status::not_found(msg),
        TmuxError::SessionAlreadyExists(_) => Status::already_exists(msg),
        TmuxError::InvalidTarget(_) | TmuxError::Validation(_) | TmuxError::ParseError { .. } => {
            Status::invalid_argument(msg)
        }
        TmuxError::TmuxNotRunning | TmuxError::CommandFailed { .. } => Status::internal(msg),
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
        let session = TmuxCommands::create_session(self, name)
            .await
            .map_err(tmux_err_to_status)?;
        Ok(Response::new(NewSessionResponse {
            name: session.name,
            windows: session.windows,
            created: session.created,
            attached: session.attached,
        }))
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
                current_path: p.current_path,
                current_command: p.current_command,
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
        let window = TmuxCommands::new_window(self, &inner.session, &inner.name)
            .await
            .map_err(tmux_err_to_status)?;
        Ok(Response::new(NewWindowResponse {
            index: window.index,
            name: window.name,
            panes: window.panes,
            active: window.active,
        }))
    }

    async fn split_window(
        &self,
        request: Request<SplitWindowRequest>,
    ) -> Result<Response<SplitWindowResponse>, Status> {
        let inner = request.into_inner();
        let pane = TmuxCommands::split_window(self, &inner.target, inner.horizontal)
            .await
            .map_err(tmux_err_to_status)?;
        Ok(Response::new(SplitWindowResponse {
            id: pane.id,
            width: pane.width,
            height: pane.height,
            active: pane.active,
            current_path: pane.current_path,
            current_command: pane.current_command,
        }))
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

    async fn capture_pane_with_options(
        &self,
        request: Request<CapturePaneWithOptionsRequest>,
    ) -> Result<Response<CapturePaneWithOptionsResponse>, Status> {
        let inner = request.into_inner();
        let opts = tmux::CaptureOptions {
            start_line: if inner.has_start_line {
                Some(inner.start_line)
            } else {
                None
            },
            end_line: if inner.has_end_line {
                Some(inner.end_line)
            } else {
                None
            },
            escape_sequences: inner.escape_sequences,
        };
        let content = TmuxCommands::capture_pane_with_options(self, &inner.target, &opts)
            .await
            .map_err(tmux_err_to_status)?;
        Ok(Response::new(CapturePaneWithOptionsResponse { content }))
    }

    async fn create_session_with_windows(
        &self,
        request: Request<CreateSessionWithWindowsRequest>,
    ) -> Result<Response<CreateSessionWithWindowsResponse>, Status> {
        let inner = request.into_inner();
        let session =
            TmuxCommands::create_session_with_windows(self, &inner.name, &inner.window_names)
                .await
                .map_err(tmux_err_to_status)?;
        Ok(Response::new(CreateSessionWithWindowsResponse {
            name: session.name,
            windows: session.windows,
            created: session.created,
            attached: session.attached,
        }))
    }

    async fn swap_panes(
        &self,
        request: Request<SwapPanesRequest>,
    ) -> Result<Response<SwapPanesResponse>, Status> {
        let inner = request.into_inner();
        TmuxCommands::swap_panes(self, &inner.src, &inner.dst)
            .await
            .map_err(tmux_err_to_status)?;
        Ok(Response::new(SwapPanesResponse {}))
    }

    async fn move_window(
        &self,
        request: Request<MoveWindowRequest>,
    ) -> Result<Response<MoveWindowResponse>, Status> {
        let inner = request.into_inner();
        TmuxCommands::move_window(self, &inner.source, &inner.destination_session)
            .await
            .map_err(tmux_err_to_status)?;
        Ok(Response::new(MoveWindowResponse {}))
    }

    async fn select_window(
        &self,
        request: Request<SelectWindowRequest>,
    ) -> Result<Response<SelectWindowResponse>, Status> {
        let target = &request.into_inner().target;
        TmuxCommands::select_window(self, target)
            .await
            .map_err(tmux_err_to_status)?;
        Ok(Response::new(SelectWindowResponse {}))
    }

    async fn select_pane(
        &self,
        request: Request<SelectPaneRequest>,
    ) -> Result<Response<SelectPaneResponse>, Status> {
        let target = &request.into_inner().target;
        TmuxCommands::select_pane(self, target)
            .await
            .map_err(tmux_err_to_status)?;
        Ok(Response::new(SelectPaneResponse {}))
    }
}

pub fn grpc_server() -> TmuxGatewayServer<TmuxGatewayServiceImpl> {
    TmuxGatewayServer::new(TmuxGatewayServiceImpl)
}
