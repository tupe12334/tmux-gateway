use tonic::{Request, Response, Status};

use super::messages::{
    CapturePaneRequest, CapturePaneResponse, CreateSessionWithWindowsRequest,
    CreateSessionWithWindowsResponse, GetOptionRequest, GetOptionResponse, KillPaneRequest,
    KillPaneResponse, KillSessionRequest, KillSessionResponse, KillWindowRequest,
    KillWindowResponse, ListOptionsRequest, ListOptionsResponse, ListPanesRequest, ListPanesResponse,
    ListWindowsRequest, ListWindowsResponse, LsRequest, LsResponse, MoveWindowRequest,
    MoveWindowResponse, NewSessionRequest, NewSessionResponse, NewWindowRequest, NewWindowResponse,
    RenameSessionRequest, RenameSessionResponse, RenameWindowRequest, RenameWindowResponse,
    SendKeysRequest, SendKeysResponse, SetOptionRequest, SetOptionResponse, SplitWindowRequest,
    SplitWindowResponse, SwapPanesRequest, SwapPanesResponse, TmuxOptionMsg, TmuxPaneMsg,
    TmuxSession, TmuxWindow,
};
use super::server::{TmuxGateway, TmuxGatewayServer};
use crate::tmux::{self, GrpcCode, OptionScope, RealTmuxExecutor, TmuxCommands, TmuxError, TmuxOption};

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

    async fn get_option(
        &self,
        target: &str,
        name: &str,
        scope: OptionScope,
    ) -> Result<String, TmuxError> {
        tmux::get_option(&RealTmuxExecutor, target, name, scope).await
    }

    async fn set_option(
        &self,
        target: &str,
        name: &str,
        value: &str,
        scope: OptionScope,
    ) -> Result<(), TmuxError> {
        tmux::set_option(&RealTmuxExecutor, target, name, value, scope).await
    }

    async fn list_options(
        &self,
        target: &str,
        scope: OptionScope,
    ) -> Result<Vec<TmuxOption>, TmuxError> {
        tmux::list_options(&RealTmuxExecutor, target, scope).await
    }
}

fn parse_scope(scope: &str) -> Result<OptionScope, Status> {
    match scope {
        "global" => Ok(OptionScope::Global),
        "session" => Ok(OptionScope::Session),
        "window" => Ok(OptionScope::Window),
        _ => Err(Status::invalid_argument(
            "scope must be one of: global, session, window",
        )),
    }
}

fn scope_to_str(scope: OptionScope) -> &'static str {
    match scope {
        OptionScope::Global => "global",
        OptionScope::Session => "session",
        OptionScope::Window => "window",
    }
}

fn tmux_err_to_status(e: TmuxError) -> Status {
    let msg = e.to_string();
    match e.grpc_code() {
        GrpcCode::NotFound => Status::not_found(msg),
        GrpcCode::AlreadyExists => Status::already_exists(msg),
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

    async fn get_option(
        &self,
        request: Request<GetOptionRequest>,
    ) -> Result<Response<GetOptionResponse>, Status> {
        let inner = request.into_inner();
        let scope = parse_scope(&inner.scope)?;
        let value = TmuxCommands::get_option(self, &inner.target, &inner.name, scope)
            .await
            .map_err(tmux_err_to_status)?;
        Ok(Response::new(GetOptionResponse { value }))
    }

    async fn set_option(
        &self,
        request: Request<SetOptionRequest>,
    ) -> Result<Response<SetOptionResponse>, Status> {
        let inner = request.into_inner();
        let scope = parse_scope(&inner.scope)?;
        TmuxCommands::set_option(self, &inner.target, &inner.name, &inner.value, scope)
            .await
            .map_err(tmux_err_to_status)?;
        Ok(Response::new(SetOptionResponse {}))
    }

    async fn list_options(
        &self,
        request: Request<ListOptionsRequest>,
    ) -> Result<Response<ListOptionsResponse>, Status> {
        let inner = request.into_inner();
        let scope = parse_scope(&inner.scope)?;
        let options = TmuxCommands::list_options(self, &inner.target, scope)
            .await
            .map_err(tmux_err_to_status)?;

        let proto_options = options
            .into_iter()
            .map(|o| TmuxOptionMsg {
                name: o.name,
                value: o.value,
                scope: scope_to_str(o.scope).to_string(),
            })
            .collect();

        Ok(Response::new(ListOptionsResponse {
            options: proto_options,
        }))
    }
}

pub fn grpc_server() -> TmuxGatewayServer<TmuxGatewayServiceImpl> {
    TmuxGatewayServer::new(TmuxGatewayServiceImpl)
}
