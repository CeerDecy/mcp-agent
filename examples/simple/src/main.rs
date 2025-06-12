fn main() {

    let s = mcp_agent::mcp_server::server::Server::new();
    s.start();

    let config = mcp_agent::agent::Config::from_file("mcp-config.toml");
    let agent = mcp_agent::agent::Agent::new_with_config(config);
}
