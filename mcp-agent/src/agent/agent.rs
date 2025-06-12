use super::config::{Config, McpConfig};
use rmcp::model::{ClientCapabilities, ClientInfo, Implementation, InitializeRequestParam};
use rmcp::service::RunningService;
use rmcp::transport::{SseClientTransport, StreamableHttpClientTransport};
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
    mcp_clients: HashMap<String, McpClient>,
    tools_clients: HashMap<String, McpClient>,
}

enum McpClient {
    StdioClient(Arc<RunningService<RoleClient, ()>>),
    RemoteClient(Arc<RunningService<RoleClient, InitializeRequestParam>>),
}

const STREAMABLE_TRANSPORT: &str = "streamable";
const STDIO_TRANSPORT: &str = "stdio";
const SSE_TRANSPORT: &str = "sse";

impl Agent {
    pub async fn new_with_config(config: Config) -> Self {
        println!("Starting MCP agent");
        let mut agent = Agent {
            mcp_clients: HashMap::new(),
            tools_clients: HashMap::new(),
        };
        agent.initialize_mcp(&config).await;
        agent
    }

    async fn initialize_mcp(&mut self, config: &Config) {
        println!("Initializing MCP clients...");
        for (name, mcp_config) in &config.mcp_servers {
            match &mcp_config.transport {
                Some(transport) => match transport.as_str() {
                    STREAMABLE_TRANSPORT => {
                        let client = Arc::new(
                            initialize_streamable_client(name, mcp_config)
                                .await
                                .unwrap(),
                        );
                        self.mcp_clients
                            .insert(name.to_owned(), McpClient::RemoteClient(client.clone()));
                        self.tools_clients
                            .insert(name.to_owned(), McpClient::RemoteClient(client.clone()));
                    }
                    STDIO_TRANSPORT => {
                        let client =
                            Arc::new(initialize_stdio_client(name, mcp_config).await.unwrap());
                        self.mcp_clients
                            .insert(name.to_owned(), McpClient::StdioClient(client.clone()));
                        self.tools_clients
                            .insert(name.to_owned(), McpClient::StdioClient(client.clone()));
                    }
                    SSE_TRANSPORT => {
                        let client =
                            Arc::new(initialize_sse_client(name, mcp_config).await.unwrap());
                        self.mcp_clients
                            .insert(name.to_owned(), McpClient::RemoteClient(client.clone()));
                        self.tools_clients
                            .insert(name.to_owned(), McpClient::RemoteClient(client.clone()));
                    }
                    _ => {
                        panic!("Unsupported transport type {}", transport);
                    }
                },
                None => {
                    let client = initialize_stdio_client(name, mcp_config).await.unwrap();
                    let c = Arc::new(client);
                    self.mcp_clients
                        .insert(name.to_owned(), McpClient::StdioClient(c.clone()));
                    self.tools_clients
                        .insert(name.to_owned(), McpClient::StdioClient(c.clone()));
                }
            }
            println!("Initialized MCP client: {}", name);
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

async fn initialize_streamable_client(
    name: &str,
    config: &McpConfig,
) -> Result<RunningService<RoleClient, InitializeRequestParam>, Box<dyn Error>> {
    let url = config.url.clone().expect("MCP url is missing");
    if url == "" {
        return Err(format!("mcp [{}] url is empty", name).into());
    }

    let transport = StreamableHttpClientTransport::from_uri(url);

    let client_info = ClientInfo {
        protocol_version: Default::default(),
        capabilities: ClientCapabilities::default(),
        client_info: Implementation {
            name: "sse client".to_string(),
            version: "1.0.0".to_string(),
        },
    };

    let client = client_info
        .serve(transport)
        .await
        .inspect_err(|err| {
            println!("mcp [{}] client error: {}", name, err);
        })
        .unwrap();

    Ok(client)
}

async fn initialize_sse_client(
    name: &str,
    config: &McpConfig,
) -> Result<RunningService<RoleClient, InitializeRequestParam>, Box<dyn Error>> {
    let url = config.url.clone().expect("MCP url is missing");
    if url == "" {
        return Err(format!("mcp [{}] url is empty", name).into());
    }
    let transport = SseClientTransport::start(url).await.unwrap();
    let client_info = ClientInfo {
        protocol_version: Default::default(),
        capabilities: ClientCapabilities::default(),
        client_info: Implementation {
            name: "sse client".to_string(),
            version: "1.0.0".to_string(),
        },
    };

    let client = client_info
        .serve(transport)
        .await
        .inspect_err(|err| {
            println!("mcp [{}] client error: {}", name, err);
        })
        .unwrap();

    Ok(client)
}
