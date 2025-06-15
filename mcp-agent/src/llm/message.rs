use async_openai::types::{
    ChatCompletionMessageToolCall, ChatCompletionRequestAssistantMessageArgs,
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs,
};
use serde::{Deserialize, Serialize};

/// Represents a conversation with a list of messages and maximum token limit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub messages: Vec<ChatCompletionRequestMessage>,
    pub max_tokens: u32,
}

impl Conversation {
    /// Creates a new empty conversation with specified maximum tokens
    pub fn new(max_tokens: u32) -> Self {
        Self {
            messages: Vec::new(),
            max_tokens,
        }
    }

    /// Creates a new conversation with a system prompt and maximum tokens
    pub fn new_with_prompt(max_tokens: u32, system_prompt: String) -> Self {
        let mut conversation = Self::new(max_tokens);
        conversation.append_system_content(system_prompt);
        conversation
    }

    /// Appends a raw message to the conversation
    pub fn append_message(&mut self, message: ChatCompletionRequestMessage) {
        self.messages.push(message);
    }

    /// Appends a user message with the given content
    pub fn append_user_content(&mut self, content: String) {
        self.messages.push(
            ChatCompletionRequestUserMessageArgs::default()
                .content(content)
                .build()
                .unwrap()
                .into(),
        )
    }

    /// Appends a system message with the given content
    pub fn append_system_content(&mut self, content: String) {
        self.messages.push(
            ChatCompletionRequestSystemMessageArgs::default()
                .content(content)
                .build()
                .unwrap()
                .into(),
        )
    }

    /// Appends an assistant message with tool calls
    pub fn append_tool_call_response(&mut self, tool_calls: &Vec<ChatCompletionMessageToolCall>) {
        self.messages.push(
            ChatCompletionRequestAssistantMessageArgs::default()
                .tool_calls(tool_calls.clone())
                .build()
                .unwrap()
                .into(),
        )
    }

    /// Appends an assistant message with the given content
    pub fn append_assistant_content(&mut self, content: String) {
        self.messages.push(
            ChatCompletionRequestAssistantMessageArgs::default()
                .content(content)
                .build()
                .unwrap()
                .into(),
        )
    }

    /// Appends a tool message with content and tool ID
    pub fn append_tool_call_content(&mut self, content: String, tool_id: String) {
        self.messages.push(
            ChatCompletionRequestToolMessageArgs::default()
                .content(content)
                .tool_call_id(tool_id)
                .build()
                .unwrap()
                .into(),
        )
    }
}

/// Represents a response from the chat model, including the conversation and optional tool calls
pub struct ChatResponse {
    pub conversation: Conversation,
    pub tool_calls: Option<Vec<ChatCompletionMessageToolCall>>,
}

impl ChatResponse {
    /// Creates a new chat response with the given conversation
    pub fn new(conversation: Conversation) -> Self {
        Self {
            conversation,
            tool_calls: None,
        }
    }
}
