use crate::agent::{Agent, Config};
use rmcp::transport::sse_server::SseServerConfig;
use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};
use rmcp::transport::{SseServer, stdio};
use rmcp::{RoleServer, Service, ServiceExt};
use std::io::Error;
use std::sync::Arc;
use std::time::Duration;

const BIND_ADDRESS: &str = "0.0.0.0:8080";
const SSE_PATH: &str = "/sse";
const MESSAGE_PATH: &str = "/message";
const STREAMABLE_PATH: &str = "/mcp";

pub struct Server {
    agent: Arc<Agent>,
}

impl Server {
    pub async fn new() -> Self {
        let agent = Arc::new(Agent::new_with_config(Config::from_file("mcp-agent.toml")).await);
        Self { agent }
    }

    pub async fn handle_sse<S, F>(&mut self, addr: &str, service_provider: F) -> Result<(), Error>
    where
        S: Service<RoleServer>,
        F: Fn(Arc<Agent>) -> S + Send + Sync + 'static,
    {
        let config = SseServerConfig {
            bind: addr.parse().unwrap(),
            sse_path: SSE_PATH.to_string(),
            post_path: MESSAGE_PATH.to_string(),
            ct: Default::default(),
            sse_keep_alive: Some(Duration::from_secs(10)),
        };

        let (sse_server, router) = SseServer::new(config);
        let listener = tokio::net::TcpListener::bind(BIND_ADDRESS).await?;
        let ct = sse_server.config.ct.child_token();
        let server = axum::serve(listener, router).with_graceful_shutdown(async move {
            ct.cancelled().await;
            println!("sse mcp_server cancelled");
        });

        tokio::spawn(async move {
            if let Err(e) = server.await {
                eprintln!("mcp_server error: {}", e);
            }
        });

        let service_provider = Arc::new(service_provider);
        let agent = self.agent.clone();
        let provider = {
            let service_provider = service_provider.clone();
            move || service_provider(agent.clone())
        };
        let ct = sse_server.with_service(provider);
        tokio::signal::ctrl_c().await?;
        ct.cancel();
        println!("ctrl-c received!");
        Ok(())
    }

    pub async fn handle_streamable<S, F>(
        &mut self,
        addr: &str,
        service_provider: F,
    ) -> Result<(), Error>
    where
        S: Service<RoleServer>,
        F: Fn(Arc<Agent>) -> S + Send + Sync + 'static,
    {
        let service_provider = Arc::new(service_provider);
        let agent = self.agent.clone();
        let provider = {
            let service_provider = service_provider.clone();
            move || service_provider(agent.clone())
        };

        let service = StreamableHttpService::new(
            provider,
            LocalSessionManager::default().into(),
            Default::default(),
        );

        let router = axum::Router::new().nest_service(STREAMABLE_PATH, service);
        let tcp_listener = tokio::net::TcpListener::bind(addr).await?;
        let _ = axum::serve(tcp_listener, router)
            .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() })
            .await;
        Ok(())
    }

    pub async fn handle_stdio<S, F>(&mut self, service_provider: F) -> Result<(), Error>
    where
        S: Service<RoleServer>,
        F: Fn(Arc<Agent>) -> S + Send + Sync + 'static,
    {
        let service_provider = Arc::new(service_provider);
        let agent = self.agent.clone();
        let provider = {
            let service_provider = service_provider.clone();
            move || service_provider(agent.clone())
        };

        provider()
            .serve(stdio())
            .await
            .inspect_err(|e| {
                println!("stdio error: {}", e);
            })
            .unwrap();

        Ok(())
    }
}
