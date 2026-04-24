use serde_json::json;

use crate::{
    agent::prompts::planner_system_prompt,
    error::BackendError,
    models::{openrouter::LlmMessage, tool::ToolPlan},
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
            LlmMessage {
                role: "system".to_string(),
                content: planner_system_prompt(),
            },
            LlmMessage {
                role: "user".to_string(),
                content: format!("User question: {user_message}"),
            },
        ];

        let plan: ToolPlan = self.llm.chat_json(messages, schema).await?;

        Ok(plan)
    }
}
