mod common;

use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs};
use common::counter::Counter;
use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParam, ClientCapabilities, ClientInfo, Implementation};
use rmcp::transport::sse_server::SseServerConfig;
use rmcp::transport::{SseClientTransport, SseServer, StreamableHttpClientTransport};
use std::borrow::Cow;
use std::io::ErrorKind::Deadlock;
use std::time::Duration;

const BIND_ADDRESS: &str = "0.0.0.0:8080";

#[tokio::main]
async fn main() {
    // sse_server().await;

    // mcp_client().await;

    // streamable_client().await;

    // openai().await;

    let s = mcp_agent::mcp_server::server::Server::new().await;
    s.start().await;

    let config = mcp_agent::agent::Config::from_file("mcp-config.toml");
    let agent = mcp_agent::agent::Agent::new_with_config(config);
}

async fn openai() {
    let config = OpenAIConfig::new()
        .with_api_key("t_19c951888f044e7b98b6585a5abfef61")
        .with_api_base("https://ai-proxy.erda.cloud/v1");

    let client = Client::with_config(config);

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u32)
        .model("gpt-4.1 [T:Azure][L:eastus2][ID:b3e7c2f1-8a5d-4f9b-9e2c-7d1b4f8e5c9a]")
        .messages([ChatCompletionRequestUserMessageArgs::default()
            .content("你好呀")
            .build()
            .unwrap()
            .into()])
        .build()
        .unwrap();

    let response = client.chat().create(request).await.unwrap();
    for choice in response.choices {
        println!(
            "{}: Role: {} Content: {:?}",
            choice.index, choice.message.role, choice.message.content
        )
    }
}

async fn streamable_client() {
    let transport =
        StreamableHttpClientTransport::from_uri("https://search-engine-7f027242.erda.cloud/sse");

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
        .inspect_err(|err| eprintln!("{:?}", err))
        .unwrap();

    let server_info = client.peer_info();
    println!("Server info: {:?}", server_info);

    let tools = client.list_tools(Default::default()).await.unwrap();
    println!("Tools: {:#?}", tools);

    let result = client
        .call_tool(CallToolRequestParam {
            name: Cow::from("search"),
            arguments: serde_json::json!({"query":"ebpf 简介","need_fetch":true})
                .as_object()
                .cloned(),
        })
        .await
        .unwrap();

    println!("Result: {:?}", result.content);

    client.cancel().await.unwrap();
}

async fn mcp_client() {
    let transport = SseClientTransport::start("https://search-engine-7f027242.erda.cloud/sse")
        .await
        .unwrap();
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
        .inspect_err(|err| eprintln!("{:?}", err))
        .unwrap();

    let server_info = client.peer_info();
    println!("Server info: {:?}", server_info);

    let tools = client.list_tools(Default::default()).await.unwrap();
    println!("Tools: {:#?}", tools);

    let result = client
        .call_tool(CallToolRequestParam {
            name: Cow::from("search"),
            arguments: serde_json::json!({"query":"ebpf简介","need_fetch":true})
                .as_object()
                .cloned(),
        })
        .await
        .unwrap();

    println!("Result: {:?}", result.content);

    client.cancel().await.unwrap();
}

async fn sse_server() {
    let config = SseServerConfig {
        bind: BIND_ADDRESS.parse().unwrap(),
        sse_path: "/sse".to_string(),
        post_path: "/message".to_string(),
        ct: Default::default(),
        sse_keep_alive: Some(Duration::from_secs(10)),
    };

    let (sse_server, router) = SseServer::new(config);
    let listener = tokio::net::TcpListener::bind(BIND_ADDRESS).await.unwrap();
    let ct = sse_server.config.ct.child_token();
    let server = axum::serve(listener, router).with_graceful_shutdown(async move {
        ct.cancelled().await;
        tracing::info!("sse mcp_server cancelled");
    });

    tokio::spawn(async move {
        if let Err(e) = server.await {
            eprintln!("mcp_server error: {}", e);
        }
    });

    let ct = sse_server.with_service(Counter::new);
    tokio::signal::ctrl_c().await.unwrap();
    ct.cancel();
    tracing::info!("ctrl-c received!");
}
