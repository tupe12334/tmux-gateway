use super::{TmuxError, TmuxPane, TmuxSession, TmuxWindow};

/// All API layers (REST, gRPC, GraphQL) must implement this trait.
/// Adding a new command here will cause a compile error in any
/// API layer that hasn't implemented it yet.
pub trait TmuxCommands {
    fn ls(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<TmuxSession>, TmuxError>> + Send;
    fn create_session(
        &self,
        name: &str,
    ) -> impl std::future::Future<Output = Result<String, TmuxError>> + Send;
    fn kill_session(
        &self,
        target: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send;
    fn kill_window(
        &self,
        target: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send;
    fn kill_pane(
        &self,
        target: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send;
    fn list_windows(
        &self,
        session: &str,
    ) -> impl std::future::Future<Output = Result<Vec<TmuxWindow>, TmuxError>> + Send;
    fn list_panes(
        &self,
        target: &str,
    ) -> impl std::future::Future<Output = Result<Vec<TmuxPane>, TmuxError>> + Send;
    fn send_keys(
        &self,
        target: &str,
        keys: &[String],
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send;
    fn rename_session(
        &self,
        target: &str,
        new_name: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send;
    fn rename_window(
        &self,
        target: &str,
        new_name: &str,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send;
    fn new_window(
        &self,
        session: &str,
        name: &str,
    ) -> impl std::future::Future<Output = Result<String, TmuxError>> + Send;
    fn split_window(
        &self,
        target: &str,
        horizontal: bool,
    ) -> impl std::future::Future<Output = Result<(), TmuxError>> + Send;
    fn capture_pane(
        &self,
        target: &str,
    ) -> impl std::future::Future<Output = Result<String, TmuxError>> + Send;
}
