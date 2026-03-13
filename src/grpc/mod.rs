mod health_service;

pub mod health_proto {
    tonic::include_proto!("health");
}

pub use health_service::health_server;
