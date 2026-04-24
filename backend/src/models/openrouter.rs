use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenRouterChatRequest {
    pub model: String,
    pub messages: Vec<LlmMessage>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<OpenRouterReasoning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<OpenRouterResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugins: Option<Vec<OpenRouterPlugin>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenRouterReasoning {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenRouterResponseFormat {
    #[serde(rename = "type")]
    pub format_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_schema: Option<OpenRouterJsonSchema>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenRouterJsonSchema {
    pub name: String,
    pub strict: bool,
    pub schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenRouterPlugin {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenRouterChatResponse {
    pub model: String,
    pub choices: Vec<OpenRouterChoice>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenRouterChoice {
    pub message: OpenRouterResponseMessage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenRouterResponseMessage {
    pub role: String,
    pub content: Option<String>,
}
