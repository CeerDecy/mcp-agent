use async_trait::async_trait;
use crate::llm::message::{ChatResponse, Conversation};

#[async_trait]
pub trait LLM: Send + Sync {
    async fn send(&self, conversation: Conversation) -> ChatResponse;
}
