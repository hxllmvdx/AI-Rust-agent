use crate::{
    agent::prompts::synthesizer_system_prompt,
    error::BackendError,
    models::{execution::ExecutionResponse, ollama::OllamaMessage},
    services::llm::LlmService,
};

#[derive(Clone)]
pub struct SynthesizerService {
    llm: LlmService,
}

impl SynthesizerService {
    pub fn new(llm: LlmService) -> Self {
        Self { llm }
    }

    pub async fn synthesize(
        &self,
        user_message: &str,
        execution: &ExecutionResponse,
    ) -> Result<String, BackendError> {
        let execution_json = serde_json::to_string_pretty(execution)?;

        let messages = vec![
            OllamaMessage {
                role: "system".to_string(),
                content: synthesizer_system_prompt(),
            },
            OllamaMessage {
                role: "user".to_string(),
                content: format!(
                    "User question:\n{}\nTool execution results:\n{}",
                    user_message, execution_json
                ),
            },
        ];

        self.llm.chat(messages).await
    }
}
