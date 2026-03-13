use super::TmuxSession;

/// All API layers (REST, gRPC, GraphQL) must implement this trait.
/// Adding a new command here will cause a compile error in any
/// API layer that hasn't implemented it yet.
pub trait TmuxCommands {
    fn ls(&self) -> impl std::future::Future<Output = Result<Vec<TmuxSession>, String>> + Send;
    fn new(&self, name: &str) -> impl std::future::Future<Output = Result<String, String>> + Send;
}
