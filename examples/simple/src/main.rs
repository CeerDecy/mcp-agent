mod tool;

use mcp_agent::mcp_server::server::Server;
use tool::calculator::Calculator;

#[tokio::main]
async fn main() {
    let mut server = Server::new().await;

    let addr = "0.0.0.0:8080";
    println!("Starting SSE server on {}", addr);
    server.handle_sse(addr, Calculator::new).await.unwrap();
}
