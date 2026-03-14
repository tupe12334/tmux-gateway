use super::{TmuxError, TmuxSession};

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
}
