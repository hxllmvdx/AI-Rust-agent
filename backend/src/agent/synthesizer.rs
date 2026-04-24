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
        let used_tools = if execution.plan.tools.is_empty() {
            "none".to_string()
        } else {
            execution
                .plan
                .tools
                .iter()
                .map(|t| format!("{}({})", t.name, t.arguments.query))
                .collect::<Vec<_>>()
                .join(", ")
        };

        let tool_results_json = serde_json::to_string_pretty(&execution.results)?;

        let user_content = format!(
            "\
        User question:
        {user_message}

        Used tools:
        {used_tools}

        Tool results:
        {tool_results_json}

        Instructions:
        - Use only the information present in the tool results.
        - If evidence is limited, say that clearly.
        - Do not invent scores or rankings.
        - Do not mention repositories unless they are present in the tool results.

        "
        );

        let messages = vec![
            OllamaMessage {
                role: "system".to_string(),

                content: synthesizer_system_prompt(),
            },
            OllamaMessage {
                role: "user".to_string(),

                content: user_content,
            },
        ];

        self.llm.chat(messages).await
    }
}
