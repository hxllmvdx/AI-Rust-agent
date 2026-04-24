use reqwest::Client;
use serde::de::DeserializeOwned;

use crate::{
    error::BackendError,
    models::openrouter::{
        LlmMessage, OpenRouterChatRequest, OpenRouterChatResponse, OpenRouterJsonSchema,
        OpenRouterPlugin, OpenRouterReasoning, OpenRouterResponseFormat,
    },
};

#[derive(Clone)]
pub struct LlmService {
    http: Client,
    base_url: String,
    api_key: String,
    model: String,
    reasoning_enabled: bool,
}

impl LlmService {
    pub fn new(base_url: String, api_key: String, model: String, reasoning_enabled: bool) -> Self {
        Self {
            http: Client::new(),
            base_url,
            api_key,
            model,
            reasoning_enabled,
        }
    }

    pub async fn chat(&self, messages: Vec<LlmMessage>) -> Result<String, BackendError> {
        let request = OpenRouterChatRequest {
            model: self.model.clone(),
            messages,
            stream: false,
            reasoning: self.reasoning_config(),
            response_format: None,
            plugins: None,
        };

        tracing::info!("sending request to openrouter: model={}", self.model);
        let response = self.send_request(request).await?;

        Self::extract_content(response)
    }

    pub async fn chat_json<T: DeserializeOwned>(
        &self,
        messages: Vec<LlmMessage>,
        schema: serde_json::Value,
    ) -> Result<T, BackendError> {
        let request = OpenRouterChatRequest {
            model: self.model.clone(),
            messages,
            stream: false,
            reasoning: None,
            response_format: Some(OpenRouterResponseFormat {
                format_type: "json_schema".to_string(),
                json_schema: Some(OpenRouterJsonSchema {
                    name: "planner_response".to_string(),
                    strict: true,
                    schema,
                }),
            }),
            plugins: Some(vec![OpenRouterPlugin {
                id: "response-healing".to_string(),
            }]),
        };

        let response = self.send_request(request).await?;
        let content = Self::extract_content(response)?;
        let parsed = serde_json::from_str::<T>(&content)?;

        Ok(parsed)
    }

    async fn send_request(
        &self,
        request: OpenRouterChatRequest,
    ) -> Result<OpenRouterChatResponse, BackendError> {
        self.http
            .post(format!(
                "{}/chat/completions",
                self.base_url.trim_end_matches('/')
            ))
            .bearer_auth(&self.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?
            .error_for_status()?
            .json::<OpenRouterChatResponse>()
            .await
            .map_err(BackendError::from)
    }

    fn extract_content(response: OpenRouterChatResponse) -> Result<String, BackendError> {
        response
            .choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .filter(|content| !content.trim().is_empty())
            .ok_or_else(|| {
                BackendError::InvalidLlmResponse(format!(
                    "empty chat completion payload from model {}",
                    response.model
                ))
            })
    }

    fn reasoning_config(&self) -> Option<OpenRouterReasoning> {
        self.reasoning_enabled
            .then_some(OpenRouterReasoning { enabled: true })
    }

    pub async fn simple_user_prompt(&self, prompt: &str) -> Result<String, BackendError> {
        tracing::info!("sending request to openrouter: model={}", self.model);
        self.chat(vec![LlmMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }])
        .await
    }
}
