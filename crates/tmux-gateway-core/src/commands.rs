use super::{
    CaptureOptions, EnvVar, HealthStatus, OptionScope, PaneLayout, RealTmuxExecutor,
    ResizeDirection, SessionDetail, TmuxEnvVar, TmuxError, TmuxOption, TmuxPane, TmuxSession,
    TmuxWindow, capture_pane, capture_pane_with_options, create_session_with_windows,
    ensure_session, ensure_window, get_option, get_server_env, get_session_detail, health_check,
    kill_pane, kill_session, kill_window, list_options, list_panes, list_server_environment,
    list_sessions, list_windows, move_window, new_session, new_window, rename_session,
    rename_window, resize_pane, select_layout, select_pane, select_window, send_keys, set_environment,
    set_option, set_server_env, show_environment, split_window, swap_panes, swap_window,
    unset_environment, unset_server_env,
};

/// All API layers (REST, gRPC, GraphQL) must implement this trait.
/// Default implementations delegate to the core free functions via `RealTmuxExecutor`.
/// Any layer can override individual methods if needed (e.g., for caching or metrics).
pub trait TmuxCommands {
    fn ls(&self) -> impl std::future::Future<Output = Result<Vec<TmuxSession>, TmuxError>> + Send {
        async { list_sessions(&RealTmuxExecutor).await }
    }
    fn create_session(
        &self,
        name: &str,
        command: Option<&str>,
    ) -> impl std::future::Future<Output = Result<TmuxSession, TmuxError>> + Send {
        async move { new_session(&RealTmuxExecutor, name, command).await }
    }
    fn kill_session(
        &self,
        target: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        async move { kill_session(&RealTmuxExecutor, target).await }
    }
    fn kill_window(
        &self,
        target: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        async move { kill_window(&RealTmuxExecutor, target).await }
    }
    fn kill_pane(
        &self,
        target: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        async move { kill_pane(&RealTmuxExecutor, target).await }
    }
    fn list_windows(
        &self,
        session: &str,
    ) -> impl std::future::Future<Output = Result<Vec<TmuxWindow>, TmuxError>> + Send {
        async move { list_windows(&RealTmuxExecutor, session).await }
    }
    fn list_panes(
        &self,
        target: &str,
    ) -> impl std::future::Future<Output = Result<Vec<TmuxPane>, TmuxError>> + Send {
        async move { list_panes(&RealTmuxExecutor, target).await }
    }
    fn send_keys(
        &self,
        target: &str,
        keys: &[String],
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        async move { send_keys(&RealTmuxExecutor, target, keys).await }
    }
    fn rename_session(
        &self,
        target: &str,
        new_name: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        async move { rename_session(&RealTmuxExecutor, target, new_name).await }
    }
    fn rename_window(
        &self,
        target: &str,
        new_name: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        async move { rename_window(&RealTmuxExecutor, target, new_name).await }
    }
    fn new_window(
        &self,
        session: &str,
        name: &str,
        command: Option<&str>,
    ) -> impl std::future::Future<Output = Result<TmuxWindow, TmuxError>> + Send {
        async move { new_window(&RealTmuxExecutor, session, name, command).await }
    }
    fn split_window(
        &self,
        target: &str,
        horizontal: bool,
        command: Option<&str>,
    ) -> impl std::future::Future<Output = Result<TmuxPane, TmuxError>> + Send {
        async move { split_window(&RealTmuxExecutor, target, horizontal, command).await }
    }
    fn capture_pane(
        &self,
        target: &str,
    ) -> impl std::future::Future<Output = Result<String, TmuxError>> + Send {
        async move { capture_pane(&RealTmuxExecutor, target).await }
    }
    fn capture_pane_with_options(
        &self,
        target: &str,
        opts: &CaptureOptions,
    ) -> impl std::future::Future<Output = Result<String, TmuxError>> + Send {
        async move { capture_pane_with_options(&RealTmuxExecutor, target, opts).await }
    }
    fn create_session_with_windows(
        &self,
        name: &str,
        window_names: &[String],
    ) -> impl std::future::Future<Output = Result<TmuxSession, TmuxError>> + Send {
        async move { create_session_with_windows(&RealTmuxExecutor, name, window_names).await }
    }
    fn swap_panes(
        &self,
        src: &str,
        dst: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        async move { swap_panes(&RealTmuxExecutor, src, dst).await }
    }
    fn swap_window(
        &self,
        src: &str,
        dst: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        async move { swap_window(&RealTmuxExecutor, src, dst).await }
    }
    fn move_window(
        &self,
        source: &str,
        destination_session: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        async move { move_window(&RealTmuxExecutor, source, destination_session).await }
    }
    fn select_window(
        &self,
        target: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        async move { select_window(&RealTmuxExecutor, target).await }
    }
    fn select_pane(
        &self,
        target: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        async move { select_pane(&RealTmuxExecutor, target).await }
    }
    fn resize_pane(
        &self,
        target: &str,
        direction: ResizeDirection,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        async move { resize_pane(&RealTmuxExecutor, target, direction).await }
    }
    fn select_layout(
        &self,
        target: &str,
        layout: PaneLayout,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        async move { select_layout(&RealTmuxExecutor, target, layout).await }
    }
    fn get_option(
        &self,
        name: &str,
        scope: OptionScope,
        target: Option<&str>,
    ) -> impl std::future::Future<Output = Result<TmuxOption, TmuxError>> + Send {
        async move { get_option(&RealTmuxExecutor, name, scope, target).await }
    }
    fn set_option(
        &self,
        name: &str,
        value: &str,
        scope: OptionScope,
        target: Option<&str>,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        async move { set_option(&RealTmuxExecutor, name, value, scope, target).await }
    }
    fn list_options(
        &self,
        scope: OptionScope,
        target: Option<&str>,
    ) -> impl std::future::Future<Output = Result<Vec<TmuxOption>, TmuxError>> + Send {
        async move { list_options(&RealTmuxExecutor, scope, target).await }
    }
    fn health_check(&self) -> impl std::future::Future<Output = HealthStatus> + Send {
        async { health_check(&RealTmuxExecutor).await }
    }
    fn ensure_session(
        &self,
        name: &str,
    ) -> impl std::future::Future<Output = Result<TmuxSession, TmuxError>> + Send {
        async move { ensure_session(&RealTmuxExecutor, name).await }
    }
    fn ensure_window(
        &self,
        session: &str,
        name: &str,
    ) -> impl std::future::Future<Output = Result<TmuxWindow, TmuxError>> + Send {
        async move { ensure_window(&RealTmuxExecutor, session, name).await }
    }
    fn get_session_detail(
        &self,
        name: &str,
    ) -> impl std::future::Future<Output = Result<SessionDetail, TmuxError>> + Send {
        async move { get_session_detail(&RealTmuxExecutor, name).await }
    }
    fn list_server_environment(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<EnvVar>, TmuxError>> + Send {
        async { list_server_environment(&RealTmuxExecutor).await }
    }
    fn get_server_env(
        &self,
        name: &str,
    ) -> impl std::future::Future<Output = Result<Option<String>, TmuxError>> + Send {
        async move { get_server_env(&RealTmuxExecutor, name).await }
    }
    fn set_server_env(
        &self,
        name: &str,
        value: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        async move { set_server_env(&RealTmuxExecutor, name, value).await }
    }
    fn unset_server_env(
        &self,
        name: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        async move { unset_server_env(&RealTmuxExecutor, name).await }
    }
    fn show_environment(
        &self,
        session: &str,
    ) -> impl std::future::Future<Output = Result<Vec<TmuxEnvVar>, TmuxError>> + Send {
        async move { show_environment(&RealTmuxExecutor, session).await }
    }
    fn set_environment(
        &self,
        session: &str,
        name: &str,
        value: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        async move { set_environment(&RealTmuxExecutor, session, name, value).await }
    }
    fn unset_environment(
        &self,
        session: &str,
        name: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send {
        async move { unset_environment(&RealTmuxExecutor, session, name).await }
    }
}
