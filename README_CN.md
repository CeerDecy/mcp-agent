# MCP Agent

MCP Agent 是一个开发框架，它将 MCP `rmcp` 工具与 LLM `async-openai` 客户端无缝集成。作为框架，它提供了一种结构化的方式来构建 MCP 工具，让开发者能够轻松地利用大语言模型的能力。框架负责处理依赖的 MCP 工具和 LLM 客户端的集成，使开发者能够专注于实现特定的业务逻辑。

## 特性

- 无缝集成 MCP 工具与 LLM 客户端
- 灵活的配置系统
- 支持多个 MCP 服务器
- 可配置的 LLM 参数
- 简单易用的接口

## 安装

你可以通过以下几种方式安装 MCP Agent：

1. 使用 `cargo add`：
   ```bash
   cargo add mcp-agent
   ```

2. 或在 `Cargo.toml` 中添加以下内容：
   ```toml
   # 从 crates.io 安装
   mcp-agent = "0.1.0"

   # 或从 GitHub 安装
   mcp-agent = { git = "https://github.com/CeerDecy/mcp-agent", branch = "main" }
   ```

## 配置

MCP Agent 使用 TOML 格式的配置文件来管理设置。配置文件通常命名为 `mcp-agent.toml`。以下是一个配置示例：

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

### 配置说明

1. **依赖的 MCP 工具配置**
   - 配置你的项目所依赖的第三方 MCP 工具
   - 每个工具可以配置自己的命令、参数和环境变量
   - 支持配置多个依赖的 MCP 工具
   - 这些工具将由框架自动管理

2. **LLM 配置**
   - `api_key`: LLM 服务的 API 密钥
   - `base_url`: LLM 服务的 API 基础 URL
   - `model`: 使用的模型名称

## 使用示例

详细的使用示例请参考 `examples` 目录中的代码。以下是基本设置的快速概览：

1. 创建配置文件：
   ```bash
   cp examples/simple/mcp-agent.toml.template mcp-agent.toml
   ```

2. 编辑配置文件，填入你的实际配置信息

3. 使用框架实现你的 MCP 工具：
   ```rust
   use mcp_agent::*;
   
   // 在这里实现你的代码
   ```

更多详细示例和最佳实践，请查看：
- `examples/simple/` - 基础使用示例
- `examples/advanced/` - 高级使用模式

## 项目结构

```
mcp-agent/
├── src/
│   ├── agent/      # Agent 核心实现
│   ├── llm/        # LLM 客户端实现
│   ├── mcp_server/ # MCP 服务器实现
│   └── lib.rs      # 库入口点
├── examples/       # 示例代码
└── Cargo.toml      # 项目依赖配置
```

## 开发

1. 克隆仓库：
   ```bash
   git clone https://github.com/CeerDecy/mcp-agent.git
   cd mcp-agent
   ```

2. 安装依赖：
   ```bash
   cargo build
   ```

3. 运行测试：
   ```bash
   cargo test
   ```

## 贡献

欢迎提交 Pull Requests 和 Issues！

## 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

Copyright (c) 2025 CeerDecy 