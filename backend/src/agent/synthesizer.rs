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
        let used_tools = execution
            .plan
            .tools
            .iter()
            .map(|t| format!("- {}({})", t.name, t.arguments.query))
            .collect::<Vec<_>>()
            .join("\n");

        let tool_results = serde_json::to_string_pretty(&execution.results)?;

        let messages = vec![
            OllamaMessage {
                role: "system".to_string(),
                content: synthesizer_system_prompt(),
            },
            OllamaMessage {
                role: "user".to_string(),
                content: format!(
                    "User question:\n{}\nUsed Tools:\n{}\nTool Results Json:\n{}",
                    user_message, used_tools, tool_results
                ),
            },
        ];

        self.llm.chat(messages).await
    }
}
