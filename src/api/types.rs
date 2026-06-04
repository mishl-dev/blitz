use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: Usage,
}

impl ChatCompletionResponse {
    pub fn new(model: String, content: String) -> Self {
        Self {
            id: format!("chatcmpl-{}", uuid_timestamp()),
            object: "chat.completion".to_string(),
            created: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            model,
            choices: vec![ChatChoice {
                index: 0,
                message: ChatMessage::assistant(content),
                finish_reason: "stop".to_string(),
            }],
            usage: Usage::default(),
        }
    }
}

fn uuid_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub object: String,
    pub data: Vec<ModelInfo>,
}

impl ModelsResponse {
    pub fn new() -> Self {
        Self {
            object: "list".to_string(),
            data: vec![ModelInfo {
                id: "llama3.1-8B".to_string(),
                object: "model".to_string(),
                created: 1700000000,
                owned_by: "meta".to_string(),
            }],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChunkChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkChoice {
    pub index: u32,
    pub delta: Delta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_assistant() {
        let msg = ChatMessage::assistant("hi there");
        assert_eq!(msg.role, "assistant");
        assert_eq!(msg.content, "hi there");
    }

    #[test]
    fn test_chat_message_user_manual() {
        let msg = ChatMessage {
            role: "user".to_string(),
            content: "hello".to_string(),
        };
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "hello");
    }

    #[test]
    fn test_chat_message_system_manual() {
        let msg = ChatMessage {
            role: "system".to_string(),
            content: "you are helpful".to_string(),
        };
        assert_eq!(msg.role, "system");
        assert_eq!(msg.content, "you are helpful");
    }

    #[test]
    fn test_chat_completion_response_new() {
        let resp =
            ChatCompletionResponse::new("llama3.1-8B".to_string(), "test response".to_string());
        assert_eq!(resp.model, "llama3.1-8B");
        assert_eq!(resp.object, "chat.completion");
        assert_eq!(resp.choices.len(), 1);
        assert_eq!(resp.choices[0].message.content, "test response");
        assert_eq!(resp.choices[0].message.role, "assistant");
        assert_eq!(resp.choices[0].finish_reason, "stop");
        assert!(resp.id.starts_with("chatcmpl-"));
    }

    #[test]
    fn test_models_response_new() {
        let resp = ModelsResponse::new();
        assert_eq!(resp.object, "list");
        assert_eq!(resp.data.len(), 1);
        assert_eq!(resp.data[0].id, "llama3.1-8B");
        assert_eq!(resp.data[0].owned_by, "meta");
    }

    #[test]
    fn test_serialize_chat_completion_request() {
        let req = ChatCompletionRequest {
            model: "llama3.1-8B".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "hello".to_string(),
            }],
            temperature: Some(0.7),
            max_tokens: Some(100),
            stream: Some(true),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("llama3.1-8B"));
        assert!(json.contains("hello"));
        assert!(json.contains("0.7"));
        assert!(json.contains("100"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_deserialize_chat_completion_request() {
        let json = r#"{"model":"llama3.1-8B","messages":[{"role":"user","content":"hi"}],"temperature":0.5,"stream":true}"#;
        let req: ChatCompletionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.model, "llama3.1-8B");
        assert_eq!(req.messages.len(), 1);
        assert_eq!(req.messages[0].role, "user");
        assert_eq!(req.temperature, Some(0.5));
        assert_eq!(req.stream, Some(true));
    }

    #[test]
    fn test_deserialize_chat_completion_request_no_stream() {
        let json = r#"{"model":"llama3.1-8B","messages":[{"role":"user","content":"hi"}]}"#;
        let req: ChatCompletionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.stream, None);
    }

    #[test]
    fn test_serialize_chat_completion_chunk() {
        let chunk = ChatCompletionChunk {
            id: "chatcmpl-123".to_string(),
            object: "chat.completion.chunk".to_string(),
            created: 1700000000,
            model: "llama3.1-8B".to_string(),
            choices: vec![ChunkChoice {
                index: 0,
                delta: Delta {
                    role: Some("assistant".to_string()),
                    content: Some("hello".to_string()),
                },
                finish_reason: Some("stop".to_string()),
            }],
        };
        let json = serde_json::to_string(&chunk).unwrap();
        assert!(json.contains("chatcmpl-123"));
        assert!(json.contains("chat.completion.chunk"));
        assert!(json.contains("hello"));
    }

    #[test]
    fn test_deserialize_chat_completion_chunk() {
        let json = r#"{"id":"chatcmpl-123","object":"chat.completion.chunk","created":1700000000,"model":"llama3.1-8B","choices":[{"index":0,"delta":{"role":"assistant","content":"hello"},"finish_reason":"stop"}]}"#;
        let chunk: ChatCompletionChunk = serde_json::from_str(json).unwrap();
        assert_eq!(chunk.id, "chatcmpl-123");
        assert_eq!(chunk.choices[0].delta.content, Some("hello".to_string()));
        assert_eq!(chunk.choices[0].finish_reason, Some("stop".to_string()));
    }

    #[test]
    fn test_deserialize_chunk_with_null_fields() {
        let json = r#"{"id":"chatcmpl-123","object":"chat.completion.chunk","created":1700000000,"model":"llama3.1-8B","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}"#;
        let chunk: ChatCompletionChunk = serde_json::from_str(json).unwrap();
        assert_eq!(chunk.choices[0].delta.role, None);
        assert_eq!(chunk.choices[0].delta.content, None);
    }
}
