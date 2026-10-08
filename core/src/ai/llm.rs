//! LLM client supporting Ollama and OpenAI-compatible APIs.

use serde::{Deserialize, Serialize};
use tracing::{info, debug, error};

use crate::config::AiProviderConfig;

/// LLM client for natural language interaction
pub struct LlmClient {
    config: AiProviderConfig,
    http: reqwest::Client,
}

impl LlmClient {
    pub fn new(config: AiProviderConfig) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
        }
    }

    /// Send a chat message and get a response
    pub async fn chat(&self, messages: &[ChatMessage], tools: &[ToolDefinition]) -> anyhow::Result<ChatResponse> {
        match self.config.provider {
            crate::config::AiProvider::Ollama => self.chat_ollama(messages, tools).await,
            crate::config::AiProvider::OpenAi | crate::config::AiProvider::Gemini => {
                self.chat_openai_compatible(messages, tools).await
            }
        }
    }

    /// Chat using Ollama API
    async fn chat_ollama(&self, messages: &[ChatMessage], _tools: &[ToolDefinition]) -> anyhow::Result<ChatResponse> {
        let url = format!("{}/api/chat", self.config.endpoint);

        let body = serde_json::json!({
            "model": self.config.model,
            "messages": messages,
            "stream": false,
        });

        let resp = self.http.post(&url)
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Ollama API error {}: {}", status, text));
        }

        let json: serde_json::Value = resp.json().await?;
        let content = json["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(ChatResponse {
            content,
            tool_calls: Vec::new(),
        })
    }

    /// Chat using OpenAI-compatible API
    async fn chat_openai_compatible(&self, messages: &[ChatMessage], tools: &[ToolDefinition]) -> anyhow::Result<ChatResponse> {
        let url = format!("{}/v1/chat/completions", self.config.endpoint);

        let mut body = serde_json::json!({
            "model": self.config.model,
            "messages": messages,
        });

        if !tools.is_empty() {
            body["tools"] = serde_json::to_value(tools)?;
        }

        let mut req = self.http.post(&url).json(&body);

        // Add API key if configured
        if let Some(ref key_id) = self.config.api_key_id {
            req = req.header("Authorization", format!("Bearer {}", key_id));
        }

        let resp = req.send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("API error {}: {}", status, text));
        }

        let json: serde_json::Value = resp.json().await?;
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(ChatResponse {
            content,
            tool_calls: Vec::new(), // Parse tool calls from response
        })
    }
}

/// A chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// A tool definition for the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// A tool call request from the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Response from the LLM
#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
}
