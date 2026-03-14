use http::Request;
use tower_http::request_id::{MakeRequestId, RequestId};
use uuid::Uuid;

/// Generates a UUID v4 for each incoming request.
#[derive(Clone)]
pub struct UuidRequestId;

impl MakeRequestId for UuidRequestId {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let id = Uuid::new_v4().to_string();
        id.parse().ok().map(RequestId::new)
    }
}
