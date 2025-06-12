use super::config::{Config, McpConfig};
use rmcp::service::RunningService;
use rmcp::{
    RoleClient,
    service::ServiceExt,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use std::collections::HashMap;
use std::error::Error;
use std::string::String;
use std::sync::Arc;
use tokio::process::Command;

pub struct Agent {
    mcp_clients: HashMap<String, Arc<RunningService<RoleClient, ()>>>,
    tools_clients: HashMap<String, Arc<RunningService<RoleClient, ()>>>,
}

impl Agent {
    pub async fn new_with_config(config: Config) -> Self {
        // config
        //     .mcp_servers
        //     .iter()
        //     .for_each(|(name, config)| match &config.transport {
        //         None => {
        //             initialize_stdio_client(name, &config).await.unwrap();
        //         }
        //         Some(transport) => {}
        //     });

        let mut agent = Agent {
            mcp_clients: HashMap::new(),
            tools_clients: HashMap::new(),
        };
        agent.initialize_mcp(&config).await;
        agent
    }

    async fn initialize_mcp(&mut self, config: &Config) {
        for (name, mcp_config) in &config.mcp_servers {
            match &mcp_config.transport {
                Some(transport) => {
                    match transport.as_str() {
                        "streamable" => {}
                        "stdio" => {
                            let client = initialize_stdio_client(name, mcp_config).await.unwrap();
                            let c = Arc::new(client);
                            self.mcp_clients.insert(name.to_owned(), c.clone());
                            self.tools_clients.insert(name.to_owned(), c.clone());
                        }
                        "sse" => {}
                        _ => {
                            panic!("Unsupported transport type {}", transport);
                        },
                    }
                }
                None => {
                    let client = initialize_stdio_client(name, mcp_config).await.unwrap();
                    let c = Arc::new(client);
                    self.mcp_clients.insert(name.to_owned(), c.clone());
                    self.tools_clients.insert(name.to_owned(), c.clone());
                }
            }
        }
    }
}

async fn initialize_stdio_client(
    name: &str,
    config: &McpConfig,
) -> Result<RunningService<RoleClient, ()>, Box<dyn Error>> {
    let command = config.command.clone();
    if command == "" {
        return Err(format!("mcp [{}] command is empty", name).into());
    }

    let client = ()
        .serve(
            TokioChildProcess::new(Command::new(command).configure(|cmd| {
                for arg in &config.args {
                    cmd.arg(arg);
                }
            }))
            .unwrap(),
        )
        .await
        .unwrap();

    Ok(client)
}

