use reqwest::Client;

use crate::{
    error::BackendError,
    models::ollama::{OllamaChatRequest, OllamaChatResponse, OllamaMessage},
};

#[derive(Clone)]
pub struct LlmService {
    http: Client,
    base_url: String,
    model: String,
}

impl LlmService {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            http: Client::new(),
            base_url,
            model,
        }
    }

    pub async fn chat(&self, messages: Vec<OllamaMessage>) -> Result<String, BackendError> {
        let request = OllamaChatRequest {
            model: self.model.clone(),
            messages,
            stream: false,
            keep_alive: "10m".to_string(),
        };

        tracing::info!("sending request to ollama: model={}", self.model);
        let response = self
            .http
            .post(format!("{}/api/chat", self.base_url))
            .json(&request)
            .send()
            .await?
            .error_for_status()?
            .json::<OllamaChatResponse>()
            .await?;

        Ok(response.message.content)
    }

    pub async fn simple_user_prompt(&self, prompt: &str) -> Result<String, BackendError> {
        tracing::info!("sending request to ollama: model={}", self.model);
        self.chat(vec![OllamaMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
        }])
        .await
    }
}
