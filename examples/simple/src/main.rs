mod tool;

use mcp_agent::mcp_server::server::Server;
use tool::tool::Tool;

#[tokio::main]
async fn main() {
    let mut server = Server::new().await;

    let addr = "0.0.0.0:8080";
    println!("Starting SSE server on {}", addr);
    server.handle_sse(addr, Tool::new).await.unwrap();
}
