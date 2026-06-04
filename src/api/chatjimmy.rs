use crate::api::types::{ChatCompletionChunk, ChatCompletionRequest, ChatCompletionResponse, ChatMessage, ChunkChoice, Delta};
use anyhow::Result;
use futures::stream::{self, BoxStream};
use futures::StreamExt;
use reqwest::Client;
use serde::Serialize;
use std::time::Duration;
use uuid::Uuid;

const CHATJIMMY_API_URL: &str = "https://chatjimmy.ai/api/chat";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Serialize)]
struct ChatJimmyRequest {
    messages: Vec<ChatMessage>,
    #[serde(rename = "chatOptions")]
    chat_options: ChatOptions,
    attachment: Option<String>,
}

#[derive(Debug, Serialize)]
struct ChatOptions {
    #[serde(rename = "selectedModel")]
    selected_model: String,
    #[serde(rename = "systemPrompt")]
    system_prompt: String,
    #[serde(rename = "topK")]
    top_k: u32,
}

pub struct ChatJimmyClient {
    client: Client,
}

impl ChatJimmyClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(CONNECT_TIMEOUT)
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client }
    }

    pub async fn complete(&self, request: &ChatCompletionRequest) -> Result<ChatCompletionResponse> {
        let content = self.fetch_content(request).await?;
        Ok(ChatCompletionResponse::new(request.model.clone(), content))
    }

    pub async fn complete_stream(&self, request: &ChatCompletionRequest) -> Result<BoxStream<'static, Result<ChatCompletionChunk, String>>> {
        let content = self.fetch_content(request).await?;
        let id = format!("chatcmpl-{}", Uuid::now_v7());
        let created = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let model = request.model.clone();

        let chunk = ChatCompletionChunk {
            id: id.clone(),
            object: "chat.completion.chunk".to_string(),
            created,
            model: model.clone(),
            choices: vec![ChunkChoice {
                index: 0,
                delta: Delta {
                    role: Some("assistant".to_string()),
                    content: Some(content),
                },
                finish_reason: Some("stop".to_string()),
            }],
        };

        let done_chunk = ChatCompletionChunk {
            id,
            object: "chat.completion.chunk".to_string(),
            created,
            model,
            choices: vec![ChunkChoice {
                index: 0,
                delta: Delta {
                    role: None,
                    content: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
        };

        Ok(stream::iter(vec![Ok(chunk), Ok(done_chunk)]).boxed())
    }

    async fn fetch_content(&self, request: &ChatCompletionRequest) -> Result<String> {
        let (system_prompt, messages) = extract_system_prompt(&request.messages);
        
        let jimmy_request = ChatJimmyRequest {
            messages,
            chat_options: ChatOptions {
                selected_model: request.model.clone(),
                system_prompt,
                top_k: 8,
            },
            attachment: None,
        };

        let response = self
            .client
            .post(CHATJIMMY_API_URL)
            .header("Content-Type", "application/json")
            .header("Accept", "*/*")
            .header("Origin", "https://chatjimmy.ai")
            .header("Referer", "https://chatjimmy.ai/")
            .json(&jimmy_request)
            .send()
            .await?;

        let text = response.text().await?;
        Ok(parse_chatjimmy_response(&text))
    }
}

fn extract_system_prompt(messages: &[ChatMessage]) -> (String, Vec<ChatMessage>) {
    let mut system_prompt = String::new();
    let mut filtered_messages = Vec::new();
    
    for msg in messages {
        if msg.role == "system" && system_prompt.is_empty() {
            system_prompt = msg.content.clone();
        } else {
            filtered_messages.push(msg.clone());
        }
    }
    
    (system_prompt, filtered_messages)
}

impl Default for ChatJimmyClient {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_chatjimmy_response(response: &str) -> String {
    if let Some(stats_start) = response.find("<|stats|>") {
        response[..stats_start].to_string()
    } else {
        response.to_string()
    }
    .trim()
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_response_with_stats() {
        let input = "Hello world<|stats|>{\"tokens\":10}<|/stats|>";
        let result = parse_chatjimmy_response(input);
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn test_parse_response_without_stats() {
        let input = "Just a plain response";
        let result = parse_chatjimmy_response(input);
        assert_eq!(result, "Just a plain response");
    }

    #[test]
    fn test_parse_response_trims_whitespace() {
        let input = "  trimmed content  <|stats|>{}<|/stats|>";
        let result = parse_chatjimmy_response(input);
        assert_eq!(result, "trimmed content");
    }

    #[test]
    fn test_parse_response_empty_stats() {
        let input = "content<|stats|><|/stats|>";
        let result = parse_chatjimmy_response(input);
        assert_eq!(result, "content");
    }

    #[test]
    fn test_parse_response_multiline() {
        let input = "Line 1\nLine 2\nLine 3<|stats|>{\"time\":1.2}<|/stats|>";
        let result = parse_chatjimmy_response(input);
        assert_eq!(result, "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_extract_system_prompt_single() {
        use crate::api::types::ChatMessage;
        let messages = vec![
            ChatMessage { role: "system".to_string(), content: "You are helpful".to_string() },
            ChatMessage { role: "user".to_string(), content: "Hello".to_string() },
        ];
        let (system, filtered) = extract_system_prompt(&messages);
        assert_eq!(system, "You are helpful");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].role, "user");
    }

    #[test]
    fn test_extract_system_prompt_none() {
        use crate::api::types::ChatMessage;
        let messages = vec![
            ChatMessage { role: "user".to_string(), content: "Hello".to_string() },
            ChatMessage { role: "assistant".to_string(), content: "Hi".to_string() },
        ];
        let (system, filtered) = extract_system_prompt(&messages);
        assert_eq!(system, "");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_extract_system_prompt_first_only() {
        use crate::api::types::ChatMessage;
        let messages = vec![
            ChatMessage { role: "system".to_string(), content: "First system".to_string() },
            ChatMessage { role: "system".to_string(), content: "Second system".to_string() },
            ChatMessage { role: "user".to_string(), content: "Hello".to_string() },
        ];
        let (system, filtered) = extract_system_prompt(&messages);
        assert_eq!(system, "First system");
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].role, "system");
        assert_eq!(filtered[0].content, "Second system");
    }
}
