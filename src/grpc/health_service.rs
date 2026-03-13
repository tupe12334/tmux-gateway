use tonic::{Request, Response, Status};

use super::health_proto::health_server::{Health, HealthServer};
use super::health_proto::{HealthCheckRequest, HealthCheckResponse, HelloRequest, HelloResponse};

pub struct HealthServiceImpl;

#[tonic::async_trait]
impl Health for HealthServiceImpl {
    async fn check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        Ok(Response::new(HealthCheckResponse {
            status: "SERVING".to_string(),
        }))
    }

    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        let name = &request.get_ref().name;
        let name = if name.is_empty() { "World" } else { name };
        Ok(Response::new(HelloResponse {
            message: format!("Hello, {}!", name),
        }))
    }
}

pub fn health_server() -> HealthServer<HealthServiceImpl> {
    HealthServer::new(HealthServiceImpl)
}
