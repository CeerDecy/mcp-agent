use mcp_agent::agent::Agent;
use rmcp::model::CallToolResult;
use rmcp::model::Content;
use rmcp::{Error as McpError, schemars, tool, ServerHandler};
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

    #[tool(description = "Calculate the sum of two numbers")]
    pub async fn calculate(
        &self,
        #[tool(aggr)] StructRequest { a, b }: StructRequest,
    ) -> Result<CallToolResult, McpError> {
        
        self.agent.send("hello ceerdecy").await;
        
        Ok(CallToolResult::success(vec![Content::text(
            (a + b).to_string(),
        )]))
    }
}

impl ServerHandler for Calculator {
    
}
