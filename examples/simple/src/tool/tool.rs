use mcp_agent::agent::Agent;
use mcp_agent::llm::message::Conversation;
use rmcp::model::Content;
use rmcp::model::{
    CallToolResult, Implementation, InitializeRequestParam, InitializeResult, ProtocolVersion,
    ServerCapabilities, ServerInfo,
};
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer, ServerHandler, schemars, tool};
use std::sync::Arc;
#[derive(Clone)]
pub struct Tool {
    agent: Arc<Agent>,
}

#[tool(tool_box)]
impl Tool {
    #[allow(dead_code)]
    pub fn new(agent: Arc<Agent>) -> Self {
        Self { agent }
    }

    #[tool(description = "Get the number of tools available")]
    pub async fn tool_count(&self) -> Result<CallToolResult, McpError> {
        let mut conversation = Conversation::new(4096);
        conversation.append_user_content(
            "how many tools you can call? and list their names, json format response: {\"data\":data}".to_string(),
        );

        let resp = self.agent.send(&mut conversation).await.unwrap();

        Ok(CallToolResult::success(vec![Content::text(resp)]))
    }

    #[tool(description = "search gas price for the province in China")]
    async fn search_gas(
        &self,
        #[tool(param)]
        #[schemars(description = "The province in China where you want to check gas prices")]
        province: String,
    ) -> Result<CallToolResult, McpError> {
        let mut conversation = Conversation::new(4096);
        conversation.append_system_content(r#"You are an AI assistant that helps users look up today's gas prices. If a Tool is called multiple times in a row, then terminate the call immediately and return success, fetch need an url param: https://www.autohome.com.cn/oil."#.to_string());
        conversation.append_user_content(
            format!(
                "please search gas price which province is {}, json format response: {}",
                province, "{\"data\": [{ \"type\": \"gas number\", \"price\": \"gas price\" }]}"
            )
            .into(),
        );

        let resp = self.agent.send(&mut conversation).await.unwrap();

        Ok(CallToolResult::success(vec![Content::text(resp)]))
    }
}

#[tool(tool_box)]
impl ServerHandler for Tool {
    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        _ = context;
        Ok(self.get_info())
    }
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This is an example mcp_server, it provides two tools".to_string()),
        }
    }
}
