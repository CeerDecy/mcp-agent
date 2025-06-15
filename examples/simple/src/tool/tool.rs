use mcp_agent::agent::Agent;
use mcp_agent::llm::message::Conversation;
use rmcp::model::Content;
use rmcp::model::{
    AnnotateAble, CallToolResult, Implementation, InitializeRequestParam, InitializeResult,
    ProtocolVersion, RawResource, Resource, ServerCapabilities, ServerInfo,
};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer, ServerHandler, schemars, tool};
use std::sync::Arc;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StructRequest {
    pub a: i32,
    pub b: i32,
}

#[derive(Clone)]
pub struct Calculator {
    agent: Arc<Agent>,
}

#[tool(tool_box)]
impl Calculator {
    #[allow(dead_code)]
    pub fn new(agent: Arc<Agent>) -> Self {
        Self { agent }
    }

    fn _create_resource_text(&self, uri: &str, name: &str) -> Resource {
        RawResource::new(uri, name.to_string()).no_annotation()
    }

    #[tool(description = "Calculate the sum of two numbers")]
    pub async fn calculate(
        &self,
        #[tool(aggr)] StructRequest { a, b }: StructRequest,
    ) -> Result<CallToolResult, McpError> {
        let mut conversation = Conversation::new(4096);
        conversation.append_user_content("how many tools you can call? json format response: {\"data\":data}".to_string());

        let resp = self.agent.send(&mut conversation).await.unwrap();

        Ok(CallToolResult::success(vec![
            Content::text((a + b).to_string()),
            Content::text(resp),
        ]))
    }

    #[tool(description = "Increment the counter by 1")]
    async fn increment(&self) -> Result<CallToolResult, McpError> {
        let mut conversation = Conversation::new(4096);
        conversation.append_user_content("how many tools you can call? json format response: {\"data\":data}".to_string());
        
        let resp = self.agent.send(&mut conversation).await.unwrap();
        Ok(CallToolResult::success(vec![Content::text(resp)]))
    }
}

#[tool(tool_box)]
impl ServerHandler for Calculator {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                // .enable_prompts()
                // .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This mcp_server provides a counter tool that can increment and decrement values. The counter starts at 0 and can be modified using the 'increment' and 'decrement' tools. Use 'get_value' to check the current count.".to_string()),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        _ = context;
        // if let Some(http_request_part) = context.extensions.get::<axum::http::request::Parts>() {
        // }
        Ok(self.get_info())
    }
}
