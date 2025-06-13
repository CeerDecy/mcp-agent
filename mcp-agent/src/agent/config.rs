use serde::Deserialize;
use std::collections::HashMap;
use std::{env, fs};

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub mcp_servers: HashMap<String, McpConfig>,

    #[serde(default)]
    pub llm: LLMConfig,
}

#[derive(Debug, Deserialize)]
pub struct McpConfig {
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub transport: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

impl Config {
    pub fn from_file(filepath: &str) -> Config {
        let content = fs::read_to_string(filepath).expect(format!("could not read config file: {}", filepath).as_str());

        toml::from_str(&content).expect("Could not parse config file")
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct LLMConfig {
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub model: String,
}
