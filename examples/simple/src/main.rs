mod tool;

use tool::calculator::Calculator;
#[tokio::main]
async fn main() {
    let s = mcp_agent::mcp_server::server::Server::new().await;
    s.register(Calculator::new).await;
    s.start().await;
}
