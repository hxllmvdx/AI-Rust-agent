use crate::{
    agent::prompts::synthesizer_system_prompt,
    error::BackendError,
    models::{execution::ExecutionResponse, ollama::OllamaMessage, sessions::ConversationMessage},
    services::llm::LlmService,
};
use serde_json::{Value, json};

const SYNTHESIZER_HISTORY_LIMIT: usize = 6;

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
        history: &[ConversationMessage],
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

        let tool_results_json = serde_json::to_string(&compact_tool_results(execution))?;
        let history_json = serde_json::to_string(&compact_history(history))?;
        let has_successful_tool = execution.results.iter().any(|result| result.success);

        let user_content = format!(
            "\
        User question:
        {user_message}

        Recent conversation:
        {history_json}

        Used tools:
        {used_tools}

        Tool results:
        {tool_results_json}

        Instructions:
        - Use only the information present in the tool results.
        - Some tools may have failed. If so, explicitly present a partial answer based on the successful tools.
        - Never claim a failed tool returned no data; say the tool failed or was unavailable.
        - If evidence is limited, say that clearly.
        - Do not invent scores or rankings.
        - Do not mention repositories unless they are present in the tool results.
        - Keep the answer compact and practical.
        - Prefer 4 short paragraphs or fewer.
        - Avoid repeating the user question or restating the tool names.

        "
        );

        if !has_successful_tool {
            return Ok(
                "I couldn't complete the full lookup because the requested tools failed. Please try again in a moment."
                    .to_string(),
            );
        }

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

fn compact_tool_results(execution: &ExecutionResponse) -> Value {
    let results = execution
        .results
        .iter()
        .map(|result| match (result.tool_name.as_str(), result.success) {
            (_, false) => json!({
                "tool": result.tool_name,
                "status": "failed",
                "error": result.error,
            }),
            ("local_knowledge_search", true) => json!({
                "tool": result.tool_name,
                "status": "ok",
                "items": extract_local_items(result.payload.as_ref()),
            }),
            ("github_search", true) => json!({
                "tool": result.tool_name,
                "status": "ok",
                "repos": extract_github_items(result.payload.as_ref()),
            }),
            _ => json!({
                "tool": result.tool_name,
                "status": "ok",
                "payload": result.payload,
            }),
        })
        .collect::<Vec<_>>();

    json!({ "results": results })
}

fn compact_history(history: &[ConversationMessage]) -> Value {
    let messages = history
        .iter()
        .filter(|message| message.role == "user" || message.role == "assistant")
        .rev()
        .take(SYNTHESIZER_HISTORY_LIMIT)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|message| {
            json!({
                "role": message.role,
                "content": truncate_message(&message.content, 300),
            })
        })
        .collect::<Vec<_>>();

    json!(messages)
}

fn extract_local_items(payload: Option<&Value>) -> Vec<Value> {
    payload
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(4)
        .map(|item| {
            json!({
                "id": item.get("id"),
                "title": item.get("title"),
                "summary": item.get("summary"),
                "pros": trim_array(item.get("pros"), 2),
                "cons": trim_array(item.get("cons"), 2),
            })
        })
        .collect()
}

fn extract_github_items(payload: Option<&Value>) -> Vec<Value> {
    payload
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(4)
        .map(|item| {
            json!({
                "full_name": item.get("full_name"),
                "description": item.get("description"),
                "language": item.get("language"),
                "stargazers_count": item.get("stargazers_count"),
                "updated_at": item.get("updated_at"),
            })
        })
        .collect()
}

fn trim_array(value: Option<&Value>, limit: usize) -> Vec<Value> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .take(limit)
        .cloned()
        .collect()
}

fn truncate_message(value: &str, limit: usize) -> String {
    let mut chars = value.chars();
    let truncated = chars.by_ref().take(limit).collect::<String>();

    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}
