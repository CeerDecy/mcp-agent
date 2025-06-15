# MCP Agent

MCP Agent is a development framework that seamlessly integrates MCP `rmcp` tools with LLM `async-openai` clients. As a framework, it provides a structured way to build MCP tools by enabling developers to easily leverage the capabilities of large language models. The framework handles the integration of dependent MCP tools and LLM clients, allowing developers to focus on implementing their specific business logic.

## Features

- Seamless integration of MCP tools with LLM clients
- Flexible configuration system
- Support for multiple MCP servers
- Configurable LLM parameters
- Simple and intuitive interface

## Installation

You can install MCP Agent in several ways:

1. Using `cargo add`:
   ```bash
   cargo add mcp-agent
   ```

2. Or add the following to your `Cargo.toml`:
   ```toml
   # From crates.io
   mcp-agent = "0.1.0"

   # Or from GitHub
   mcp-agent = { git = "https://github.com/CeerDecy/mcp-agent", branch = "main" }
   ```

## Configuration

MCP Agent uses a TOML configuration file to manage settings. The configuration file is typically named `mcp-agent.toml`. Here's an example configuration:

```toml
[mcp_servers.searxng]
command = "uvx"
args = ["mcp-searxng"]
[mcp_servers.searxng.env]
SEARXNG_URL = "https://searxng.example.com"

[mcp_servers.fetch]
command = "uvx"
args = ["mcp-server-fetch", "--ignore-robots-txt"]

[llm]
api_key = "your_api_key"
base_url = "https://api.openai.com/v1/"
model = "gpt-4.1"
```

### Configuration Details

1. **Dependent MCP Tools Configuration**
   - Configure the third-party MCP tools that your project depends on
   - Each tool can have its own command, arguments, and environment variables
   - Support for multiple dependent MCP tools
   - These tools will be automatically managed by the framework

2. **LLM Configuration**
   - `api_key`: API key for the LLM service
   - `base_url`: Base URL for the LLM API
   - `model`: Name of the model to use

## Usage Example

For detailed usage examples, please refer to the code in the `examples` directory. Here's a quick overview of the basic setup:

1. Create configuration file:
   ```bash
   cp examples/simple/mcp-agent.toml.template mcp-agent.toml
   ```

2. Edit the configuration file with your actual settings

3. Implement your MCP tool using the framework:
   ```rust
   use mcp_agent::*;
   
   // Your implementation here
   ```

For more detailed examples and best practices, please check:
- `examples/simple/` - Basic usage example
- `examples/advanced/` - Advanced usage patterns

## Project Structure

```
mcp-agent/
├── src/
│   ├── agent/      # Core agent implementation
│   ├── llm/        # LLM client implementation
│   ├── mcp_server/ # MCP server implementation
│   └── lib.rs      # Library entry point
├── examples/       # Example code
└── Cargo.toml      # Project dependencies
```

## Development

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/mcp-agent.git
   cd mcp-agent
   ```

2. Install dependencies:
   ```bash
   cargo build
   ```

3. Run tests:
   ```bash
   cargo test
   ```

## Contributing

Pull requests and issues are welcome!

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

Copyright (c) 2025 CeerDecy
