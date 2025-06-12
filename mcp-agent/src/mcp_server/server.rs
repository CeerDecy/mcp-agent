use crate::agent::{Agent, Config};
use rmcp::transport::SseServer;
use rmcp::transport::sse_server::SseServerConfig;
use rmcp::{RoleServer, Service};
use std::sync::Arc;
use std::time::Duration;

const BIND_ADDRESS: &str = "0.0.0.0:8080";

pub struct Server {
    agent: Arc<Agent>,
}

impl Server {
    pub async fn new() -> Self {
        let agent = Arc::new(Agent::new_with_config(Config::from_file("mcp-agent.toml")).await);
        Self { agent }
    }

    pub async fn register<S, F>(&self, service_provider: F)
    where
        S: Service<RoleServer>,
        F: Fn(Arc<Agent>) -> S + Send + 'static,
    {
        // let provider = service_provider(self.agent);
        // sse_server.with_service(provider);
    }

    pub async fn start(&self) {
        println!("mcp_server start");
    }
}
