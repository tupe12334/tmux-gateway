use super::{
    CaptureOptions, HealthStatus, RealTmuxExecutor, ResizeDirection, TmuxError, TmuxPane,
    TmuxSession, TmuxWindow, capture_pane, capture_pane_with_options, create_session_with_windows,
    health_check, kill_pane, kill_session, kill_window, list_panes, list_sessions, list_windows,
    move_window, new_session, new_window, rename_session, rename_window, resize_pane, select_pane,
    select_window, send_keys, split_window, swap_panes,
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
    ) -> impl std::future::Future<Output = Result<TmuxSession, TmuxError>> + Send {
        async move { new_session(&RealTmuxExecutor, name).await }
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
    ) -> impl std::future::Future<Output = Result<TmuxWindow, TmuxError>> + Send {
        async move { new_window(&RealTmuxExecutor, session, name).await }
    }
    fn split_window(
        &self,
        target: &str,
        horizontal: bool,
    ) -> impl std::future::Future<Output = Result<TmuxPane, TmuxError>> + Send {
        async move { split_window(&RealTmuxExecutor, target, horizontal).await }
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
    fn health_check(&self) -> impl std::future::Future<Output = HealthStatus> + Send {
        async { health_check(&RealTmuxExecutor).await }
    }
}
