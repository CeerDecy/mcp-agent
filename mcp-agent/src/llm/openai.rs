use crate::llm::llm::LLM;
use crate::llm::message::{ChatResponse, Conversation};
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::{ChatCompletionTool, CreateChatCompletionRequestArgs, ResponseFormat};
use async_trait::async_trait;

const BASE_URL: &str = "https://api.openai.com/v1";

/// Represents an OpenAI client for handling chat completions
#[derive(Debug)]
pub struct OpenAi {
    api_key: String,
    base_url: String,
    model: String,

    tools: Vec<ChatCompletionTool>,
    client: Client<OpenAIConfig>,
}

impl OpenAi {
    /// Creates a new instance of OpenAi with default values
    pub fn new() -> Self {
        Self {
            api_key: Default::default(),
            base_url: BASE_URL.to_string(),
            model: Default::default(),

            tools: Vec::new(),
            client: Client::new(),
        }
    }

    /// Applies a single configuration option to the OpenAi instance
    pub fn with_option(&mut self, opt: OpenAiOption) -> &Self {
        opt(self);
        self
    }

    /// Applies multiple configuration options to the OpenAi instance
    pub fn with_options(&mut self, opts: Vec<OpenAiOption>) -> &Self {
        for func in opts {
            func(self);
        }
        self
    }

    /// Builds the OpenAi client with the configured settings
    pub fn build(&mut self) -> &Self {
        let config = OpenAIConfig::new()
            .with_api_key(self.api_key.clone())
            .with_api_base(self.base_url.clone());

        let client = Client::with_config(config);

        self.client = client;

        self
    }
}

#[async_trait]
impl LLM for OpenAi {
    /// Sends a conversation to OpenAI and returns the chat response
    async fn send(&self, conversation: Conversation) -> ChatResponse {
        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(conversation.max_tokens)
            .model(&self.model)
            .messages(conversation.messages.clone())
            .tools(self.tools.clone())
            .response_format(ResponseFormat::JsonObject)
            .build()
            .unwrap();

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .inspect_err(|err| {
                println!("Failed to send CreateChatCompletionRequest: {}", err);
            })
            .unwrap();

        println!("Sending response {:?}", &response);

        let mut resp = ChatResponse::new(conversation.clone());

        for choice in response.choices {
            if let Some(content) = &choice.message.content {
                resp.conversation.append_assistant_content(content.clone());
            }

            if let Some(tools) = &choice.message.tool_calls {
                if let Some(exists) = &mut resp.tool_calls {
                    exists.extend_from_slice(tools);
                } else {
                    resp.tool_calls = Some(tools.clone());
                }
            }
        }

        resp
    }
}

type OpenAiOption = Box<dyn FnOnce(&mut OpenAi)>;

/// Creates an option to set the API key
pub fn with_api_key(api_key: &str) -> OpenAiOption {
    let api_key = api_key.to_string();
    Box::new(move |openai| openai.api_key = api_key)
}

/// Creates an option to set the model name
pub fn with_model(model: &str) -> OpenAiOption {
    let model = model.to_string();
    Box::new(move |openai| openai.model = model)
}

/// Creates an option to set the base URL
pub fn with_base_url(base_url: &str) -> OpenAiOption {
    let base_url = base_url.trim_matches('/').to_string();
    Box::new(move |openai| openai.base_url = base_url)
}

/// Creates an option to set the tools for chat completion
pub fn with_tools(tools: Vec<ChatCompletionTool>) -> OpenAiOption {
    Box::new(move |openai| openai.tools = tools)
}
