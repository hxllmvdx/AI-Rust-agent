use serde_json::json;

use crate::{
    agent::prompts::planner_system_prompt,
    error::BackendError,
    models::{ollama::OllamaMessage, tool::ToolPlan},
    services::llm::LlmService,
};

#[derive(Clone)]
pub struct PlannerService {
    llm: LlmService,
}

impl PlannerService {
    pub fn new(llm: LlmService) -> Self {
        Self { llm }
    }

    pub async fn plan(&self, user_message: &str) -> Result<ToolPlan, BackendError> {
        let schema = json!({
            "type": "object",
            "properties": {
                "need_tools": { "type": "boolean" },
                "tools": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "arguments": {
                                "type": "object",
                                "properties": {
                                    "query": { "type": "string" }
                                },
                                "required": ["query"]
                            }
                        },
                        "required": ["name", "arguments"]
                    }
                }
            },
            "required": ["need_tools", "tools"]
        });

        let messages = vec![
            OllamaMessage {
                role: "system".to_string(),
                content: planner_system_prompt(),
            },
            OllamaMessage {
                role: "user".to_string(),
                content: format!("User question: {user_message}"),
            },
        ];

        let mut plan: ToolPlan = self.llm.chat_json(messages, schema).await?;

        if !should_allow_github(user_message) {
            plan.tools.retain(|tool| tool.name != "github_search");
            plan.need_tools = !plan.tools.is_empty();
        }

        Ok(plan)
    }
}

fn should_allow_github(user_message: &str) -> bool {
    let msg = user_message.to_lowercase();

    let keywords = [
        "github",
        "repo",
        "repos",
        "repository",
        "repositories",
        "active",
        "current",
        "latest",
        "stars",
        "updated",
        "recent",
    ];

    keywords.iter().any(|k| msg.contains(k))
}
