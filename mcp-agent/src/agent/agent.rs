use super::config::{Config, McpConfig};
use crate::llm::llm::LLM;
use crate::llm::message::Conversation;
use crate::llm::openai::{OpenAi, with_api_key, with_base_url, with_model, with_tools};
use async_openai::types::ChatCompletionRequestAssistantMessageContent::Text;
use async_openai::types::ChatCompletionRequestMessage::Assistant;
use async_openai::types::{
    ChatCompletionMessageToolCall, ChatCompletionTool, ChatCompletionToolArgs,
    ChatCompletionToolType, FunctionObjectArgs,
};
use rmcp::model::{
    CallToolRequestParam, CallToolResult, ClientCapabilities, ClientInfo, Implementation,
    InitializeRequestParam, JsonObject, ListToolsResult,
};
use rmcp::service::RunningService;
use rmcp::transport::{SseClientTransport, StreamableHttpClientTransport};
use rmcp::{
    RoleClient,
    service::ServiceExt,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::string::String;
use std::sync::Arc;
use tokio::process::Command;

/// Represents an agent that manages MCP clients and LLM interactions
pub struct Agent {
    config: Config,
    mcp_clients: HashMap<String, McpClient>,
    tools_clients: HashMap<String, McpClient>,
    llm_client: Option<Arc<dyn LLM>>,
}

/// Enum representing different types of MCP clients
enum McpClient {
    /// Client using standard I/O communication
    StdioClient(Arc<RunningService<RoleClient, ()>>),
    /// Client using Server-Sent Events (SSE) communication
    SseClient(Arc<RunningService<RoleClient, InitializeRequestParam>>),
    /// Client using streamable HTTP communication
    StreamableClient(Arc<RunningService<RoleClient, InitializeRequestParam>>),
}

impl McpClient {
    /// Lists available tools from the MCP client
    async fn list_tools(&self) -> Result<ListToolsResult, Box<dyn Error>> {
        match self {
            McpClient::StdioClient(client) => {
                let tools = client.list_tools(Default::default()).await?;
                Ok(tools)
            }
            McpClient::SseClient(client) => {
                let tools = client.list_tools(Default::default()).await?;
                Ok(tools)
            }
            McpClient::StreamableClient(client) => {
                let tools = client.list_tools(Default::default()).await?;
                Ok(tools)
            }
        }
    }

    /// Calls dependence tool with the MCP client
    async fn call_tool(
        &self,
        request_param: CallToolRequestParam,
    ) -> Result<CallToolResult, Box<dyn Error>> {
        match self {
            McpClient::StdioClient(client) => Ok(client.call_tool(request_param).await?),
            McpClient::SseClient(client) => Ok(client.call_tool(request_param).await?),
            McpClient::StreamableClient(client) => Ok(client.call_tool(request_param).await?),
        }
    }
}

const STREAMABLE_TRANSPORT: &str = "streamable";
const STDIO_TRANSPORT: &str = "stdio";
const SSE_TRANSPORT: &str = "sse";

impl Agent {
    /// Creates a new agent with the configuration
    pub async fn new_with_config(config: Config) -> Self {
        println!("Starting MCP agent");
        let mut agent = Agent {
            config,
            mcp_clients: HashMap::new(),
            tools_clients: HashMap::new(),
            llm_client: None,
        };

        agent.initialize().await;

        agent
    }

    /// Sends a conversation to the LLM and returns the response
    pub async fn send(&self, conversation: &mut Conversation) -> Result<String, Box<dyn Error>> {
        self.send_llm(conversation).await
    }

    /// Internal method to send conversation to LLM and handle tool calls
    async fn send_llm(&self, conversation: &mut Conversation) -> Result<String, Box<dyn Error>> {
        println!("Sending mcp command {:?}", conversation);
        let response = self
            .llm_client
            .as_ref()
            .unwrap()
            .send(conversation.clone())
            .await;

        if response.tool_calls.is_none() {
            let message = response.conversation.messages.last().unwrap();
            if let Assistant(msg) = message {
                if let Text(content) = msg.content.clone().unwrap() {
                    return Ok(content);
                }
            }
            return Ok("success".to_string());
        }
        let toolcalls = response.tool_calls.unwrap();
        conversation.append_tool_call_response(&toolcalls);

        self.handle_tool_calls(toolcalls, conversation).await?;

        Box::pin(self.send_llm(conversation)).await
    }

    /// Initializes the agent by setting up MCP and LLM clients
    pub async fn initialize(&mut self) {
        self.initialize_mcp()
            .await
            .inspect_err(|err| {
                println!("Failed to initialize MCP client: {}", err);
            })
            .unwrap();

        self.initialize_llm()
            .await
            .inspect_err(|err| {
                println!("Failed to initialize LLM client: {}", err);
            })
            .unwrap();
    }

    /// Initializes the LLM client with configuration
    async fn initialize_llm(&mut self) -> Result<(), Box<dyn Error>> {
        let mut llm = OpenAi::new();
        llm.with_options(vec![
            with_api_key(self.config.llm.api_key.as_str()),
            with_model(self.config.llm.model.as_str()),
        ]);

        if !self.config.llm.base_url.is_empty() {
            llm.with_option(with_base_url(self.config.llm.base_url.as_str()));
        }

        let tools = self.list_tools().await?;
        if tools.len() > 0 {
            llm.with_option(with_tools(tools));
        }

        llm.build();

        self.llm_client = Some(Arc::new(llm));
        Ok(())
    }

    /// Initializes MCP clients based on configuration
    async fn initialize_mcp(&mut self) -> Result<(), Box<dyn Error>> {
        println!("Initializing MCP clients...");
        for (name, mcp_config) in &self.config.mcp_servers {
            match &mcp_config.transport {
                Some(transport) => match transport.as_str() {
                    STREAMABLE_TRANSPORT => {
                        let client =
                            Arc::new(initialize_streamable_client(name, mcp_config).await?);
                        self.mcp_clients
                            .insert(name.to_owned(), McpClient::StreamableClient(client.clone()));
                        self.tools_clients
                            .insert(name.to_owned(), McpClient::StreamableClient(client.clone()));
                    }
                    STDIO_TRANSPORT => {
                        let client = Arc::new(initialize_stdio_client(name, mcp_config).await?);
                        self.mcp_clients
                            .insert(name.to_owned(), McpClient::StdioClient(client.clone()));
                        self.tools_clients
                            .insert(name.to_owned(), McpClient::StdioClient(client.clone()));
                    }
                    SSE_TRANSPORT => {
                        let client = Arc::new(initialize_sse_client(name, mcp_config).await?);
                        self.mcp_clients
                            .insert(name.to_owned(), McpClient::SseClient(client.clone()));
                        self.tools_clients
                            .insert(name.to_owned(), McpClient::SseClient(client.clone()));
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
        Ok(())
    }

    /// Lists all available tools from MCP clients and build Vec<ChatCompletionTool>
    async fn list_tools(&mut self) -> Result<Vec<ChatCompletionTool>, Box<dyn Error>> {
        let mut res = Vec::new();
        for (_, client) in &self.mcp_clients {
            let tools = client.list_tools().await.inspect_err(|err| {
                println!("Error listing tools: {}", err);
            })?;
            for tool in tools.tools {
                res.push(
                    ChatCompletionToolArgs::default()
                        .r#type(ChatCompletionToolType::Function)
                        .function(
                            FunctionObjectArgs::default()
                                .name(tool.name)
                                .description(tool.description.unwrap())
                                .parameters(convert_json_object(tool.input_schema))
                                .build()?,
                        )
                        .build()?,
                )
            }
        }
        Ok(res)
    }

    /// Handles tool calls from the LLM response
    async fn handle_tool_calls(
        &self,
        toolcalls: Vec<ChatCompletionMessageToolCall>,
        conversation: &mut Conversation,
    ) -> Result<(), Box<dyn Error>> {
        for call in toolcalls {
            let name: String = call.function.name;
            let arguments: String = call.function.arguments;

            println!("deal tool call: {}, arguments: {:?}", &name, arguments);

            if let Some(client) = self.tools_clients.get(&name) {
                let result = client
                    .call_tool(CallToolRequestParam {
                        name: name.into(),
                        arguments: serde_json::from_str::<Value>(&arguments)?
                            .as_object()
                            .cloned(),
                    })
                    .await?;

                let resp = serde_json::to_string(&result)?;

                conversation.append_tool_call_content(resp, call.id);
            }
        }

        Ok(())
    }
}

/// Converts a JsonObject to a serde_json Value
fn convert_json_object(obj: Arc<JsonObject>) -> Option<Value> {
    let option_value = match Arc::try_unwrap(obj) {
        Ok(json_obj) => Some(Value::Object(json_obj)),
        Err(arc) => Some(Value::Object((*arc).clone())),
    };
    option_value
}

/// Initializes a stdio-based MCP client
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

/// Initializes a streamable HTTP-based MCP client
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

/// Initializes an SSE-based MCP client
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
