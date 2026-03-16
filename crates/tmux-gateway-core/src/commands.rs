use super::{
    CaptureOptions, EnvVar, HealthStatus, OptionScope, PaneLayout, RealTmuxExecutor,
    ResizeDirection, SessionDetail, TmuxBuffer, TmuxEnvVar, TmuxError, TmuxMessage, TmuxOption,
    TmuxPane, TmuxSession, TmuxWindow, capture_pane, capture_pane_with_options,
    create_session_with_windows, delete_buffer, ensure_session, ensure_window, get_buffer,
    get_option, get_server_env, get_session_detail, has_session, health_check, kill_pane,
    kill_session, kill_window, list_buffers, list_options, list_panes, list_server_environment,
    list_sessions, list_windows, move_window, new_session, new_window, paste_buffer,
    rename_session, rename_window, resize_pane, respawn_pane, respawn_window, select_layout,
    select_pane, select_window, send_keys, set_buffer, set_environment, set_option, set_server_env,
    show_environment, show_messages, split_window, swap_panes, swap_window, unset_environment,
    unset_server_env,
};
use crate::validation::{PaneTarget, SessionName, WindowTarget};

/// All API layers (REST, gRPC, GraphQL) must implement this trait.
/// Default implementations delegate to the core free functions via `RealTmuxExecutor`.
/// Any layer can override individual methods if needed (e.g., for caching or metrics).
///
/// String parameters are validated and converted to domain newtypes at this boundary.
/// Override [`executor`](TmuxCommands::executor) to target a specific tmux server socket.
pub trait TmuxCommands {
    /// Returns the executor used by all default method implementations.
    ///
    /// Override this to target a specific tmux server socket:
    /// ```ignore
    /// fn executor(&self) -> RealTmuxExecutor {
    ///     RealTmuxExecutor::with_socket("/tmp/my-tmux-socket")
    /// }
    /// ```
    fn executor(&self) -> RealTmuxExecutor {
        RealTmuxExecutor::new()
    }

    fn ls(&self) -> impl std::future::Future<Output = Result<Vec<TmuxSession>, TmuxError>> + Send {
        let executor = self.executor();
        async move { list_sessions(&executor).await }
    }
    fn create_session(
        &self,
        name: &str,
        command: Option<&str>,
    ) -> impl std::future::Future<Output = Result<TmuxSession, TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let name = SessionName::try_from(name)?;
            new_session(&executor, &name, command).await
        }
    }
    fn kill_session(
        &self,
        target: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let target = SessionName::try_from(target)?;
            kill_session(&executor, &target).await
        }
    }
    fn has_session(
        &self,
        target: &str,
    ) -> impl std::future::Future<Output = Result<bool, TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let target = SessionName::try_from(target)?;
            has_session(&executor, &target).await
        }
    }
    fn kill_window(
        &self,
        target: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let target = WindowTarget::try_from(target)?;
            kill_window(&executor, &target).await
        }
    }
    fn kill_pane(
        &self,
        target: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let target = PaneTarget::try_from(target)?;
            kill_pane(&executor, &target).await
        }
    }
    fn list_windows(
        &self,
        session: &str,
    ) -> impl std::future::Future<Output = Result<Vec<TmuxWindow>, TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let session = SessionName::try_from(session)?;
            list_windows(&executor, &session).await
        }
    }
    fn list_panes(
        &self,
        target: &str,
    ) -> impl std::future::Future<Output = Result<Vec<TmuxPane>, TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let target = WindowTarget::try_from(target)?;
            list_panes(&executor, &target).await
        }
    }
    fn send_keys(
        &self,
        target: &str,
        keys: &[String],
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let target = PaneTarget::try_from(target)?;
            send_keys(&executor, &target, keys).await
        }
    }
    fn rename_session(
        &self,
        target: &str,
        new_name: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let target = SessionName::try_from(target)?;
            let new_name = SessionName::try_from(new_name)?;
            rename_session(&executor, &target, &new_name).await
        }
    }
    fn rename_window(
        &self,
        target: &str,
        new_name: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let target = WindowTarget::try_from(target)?;
            rename_window(&executor, &target, new_name).await
        }
    }
    fn new_window(
        &self,
        session: &str,
        name: &str,
        command: Option<&str>,
    ) -> impl std::future::Future<Output = Result<TmuxWindow, TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let session = SessionName::try_from(session)?;
            new_window(&executor, &session, name, command).await
        }
    }
    fn split_window(
        &self,
        target: &str,
        horizontal: bool,
        command: Option<&str>,
    ) -> impl std::future::Future<Output = Result<TmuxPane, TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let target = PaneTarget::try_from(target)?;
            split_window(&executor, &target, horizontal, command).await
        }
    }
    fn capture_pane(
        &self,
        target: &str,
    ) -> impl std::future::Future<Output = Result<String, TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let target = PaneTarget::try_from(target)?;
            capture_pane(&executor, &target).await
        }
    }
    fn capture_pane_with_options(
        &self,
        target: &str,
        opts: &CaptureOptions,
    ) -> impl std::future::Future<Output = Result<String, TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let target = PaneTarget::try_from(target)?;
            capture_pane_with_options(&executor, &target, opts).await
        }
    }
    fn create_session_with_windows(
        &self,
        name: &str,
        window_names: &[String],
    ) -> impl std::future::Future<Output = Result<TmuxSession, TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let name = SessionName::try_from(name)?;
            create_session_with_windows(&executor, &name, window_names).await
        }
    }
    fn swap_panes(
        &self,
        src: &str,
        dst: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let src = PaneTarget::try_from(src)?;
            let dst = PaneTarget::try_from(dst)?;
            swap_panes(&executor, &src, &dst).await
        }
    }
    fn swap_window(
        &self,
        src: &str,
        dst: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let src = WindowTarget::try_from(src)?;
            let dst = WindowTarget::try_from(dst)?;
            swap_window(&executor, &src, &dst).await
        }
    }
    fn move_window(
        &self,
        source: &str,
        destination_session: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let source = WindowTarget::try_from(source)?;
            let destination_session = SessionName::try_from(destination_session)?;
            move_window(&executor, &source, &destination_session).await
        }
    }
    fn select_window(
        &self,
        target: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let target = WindowTarget::try_from(target)?;
            select_window(&executor, &target).await
        }
    }
    fn select_pane(
        &self,
        target: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let target = PaneTarget::try_from(target)?;
            select_pane(&executor, &target).await
        }
    }
    fn resize_pane(
        &self,
        target: &str,
        direction: ResizeDirection,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let target = PaneTarget::try_from(target)?;
            resize_pane(&executor, &target, direction).await
        }
    }
    fn select_layout(
        &self,
        target: &str,
        layout: PaneLayout,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let target = WindowTarget::try_from(target)?;
            select_layout(&executor, &target, layout).await
        }
    }
    fn get_option(
        &self,
        name: &str,
        scope: OptionScope,
        target: Option<&str>,
    ) -> impl std::future::Future<Output = Result<TmuxOption, TmuxError>> + Send {
        let executor = self.executor();
        async move { get_option(&executor, name, scope, target).await }
    }
    fn set_option(
        &self,
        name: &str,
        value: &str,
        scope: OptionScope,
        target: Option<&str>,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        let executor = self.executor();
        async move { set_option(&executor, name, value, scope, target).await }
    }
    fn list_options(
        &self,
        scope: OptionScope,
        target: Option<&str>,
    ) -> impl std::future::Future<Output = Result<Vec<TmuxOption>, TmuxError>> + Send {
        let executor = self.executor();
        async move { list_options(&executor, scope, target).await }
    }
    fn health_check(&self) -> impl std::future::Future<Output = HealthStatus> + Send {
        let executor = self.executor();
        async move { health_check(&executor).await }
    }
    fn ensure_session(
        &self,
        name: &str,
    ) -> impl std::future::Future<Output = Result<TmuxSession, TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let name = SessionName::try_from(name)?;
            ensure_session(&executor, &name).await
        }
    }
    fn ensure_window(
        &self,
        session: &str,
        name: &str,
    ) -> impl std::future::Future<Output = Result<TmuxWindow, TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let session = SessionName::try_from(session)?;
            ensure_window(&executor, &session, name).await
        }
    }
    fn get_session_detail(
        &self,
        name: &str,
    ) -> impl std::future::Future<Output = Result<SessionDetail, TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let name = SessionName::try_from(name)?;
            get_session_detail(&executor, &name).await
        }
    }
    fn list_server_environment(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<EnvVar>, TmuxError>> + Send {
        let executor = self.executor();
        async move { list_server_environment(&executor).await }
    }
    fn get_server_env(
        &self,
        name: &str,
    ) -> impl std::future::Future<Output = Result<Option<String>, TmuxError>> + Send {
        let executor = self.executor();
        async move { get_server_env(&executor, name).await }
    }
    fn set_server_env(
        &self,
        name: &str,
        value: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        let executor = self.executor();
        async move { set_server_env(&executor, name, value).await }
    }
    fn unset_server_env(
        &self,
        name: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        let executor = self.executor();
        async move { unset_server_env(&executor, name).await }
    }
    fn list_buffers(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<TmuxBuffer>, TmuxError>> + Send {
        let executor = self.executor();
        async move { list_buffers(&executor).await }
    }
    fn get_buffer(
        &self,
        name: Option<&str>,
    ) -> impl std::future::Future<Output = Result<String, TmuxError>> + Send {
        let executor = self.executor();
        async move { get_buffer(&executor, name).await }
    }
    fn set_buffer(
        &self,
        name: Option<&str>,
        content: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        let executor = self.executor();
        async move { set_buffer(&executor, name, content).await }
    }
    fn paste_buffer(
        &self,
        target: &str,
        name: Option<&str>,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let target = PaneTarget::try_from(target)?;
            paste_buffer(&executor, &target, name).await
        }
    }
    fn delete_buffer(
        &self,
        name: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        let executor = self.executor();
        async move { delete_buffer(&executor, name).await }
    }
    fn show_environment(
        &self,
        session: &str,
    ) -> impl std::future::Future<Output = Result<Vec<TmuxEnvVar>, TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let session = SessionName::try_from(session)?;
            show_environment(&executor, &session).await
        }
    }
    fn set_environment(
        &self,
        session: &str,
        name: &str,
        value: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let session = SessionName::try_from(session)?;
            set_environment(&executor, &session, name, value).await
        }
    }
    fn unset_environment(
        &self,
        session: &str,
        name: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let session = SessionName::try_from(session)?;
            unset_environment(&executor, &session, name).await
        }
    }
    fn respawn_pane(
        &self,
        target: &str,
        command: Option<&str>,
        kill_existing: bool,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let target = PaneTarget::try_from(target)?;
            respawn_pane(&executor, &target, command, kill_existing).await
        }
    }
    fn respawn_window(
        &self,
        target: &str,
        command: Option<&str>,
        kill_existing: bool,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        let executor = self.executor();
        async move {
            let target = WindowTarget::try_from(target)?;
            respawn_window(&executor, &target, command, kill_existing).await
        }
    }
    fn show_messages(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<TmuxMessage>, TmuxError>> + Send {
        let executor = self.executor();
        async move { show_messages(&executor).await }
    }
}
