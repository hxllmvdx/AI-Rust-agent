use serde_json::to_value;

use crate::{
    error::BackendError,
    models::{
        execution::{ExecutionResponse, ToolExecutionResult},
        sessions::ConversationMessage,
        tool::ToolCall,
    },
    services::session_store::SessionStore,
    tools::{github::GitHubTool, local_data::LocalKnowledgeTool},
};

use super::{planner::PlannerService, synthesizer::SynthesizerService};

#[derive(Clone)]
pub struct OrchestratorService {
    planner: PlannerService,
    synthesizer: SynthesizerService,
    local_tool: LocalKnowledgeTool,
    github_tool: GitHubTool,
    sessions: SessionStore,
}

impl OrchestratorService {
    pub fn new(
        planner: PlannerService,
        synthesizer: SynthesizerService,
        local_tool: LocalKnowledgeTool,
        github_tool: GitHubTool,
        sessions: SessionStore,
    ) -> Self {
        Self {
            planner,
            synthesizer,
            local_tool,
            github_tool,
            sessions,
        }
    }

    pub async fn execute(&self, user_message: &str) -> Result<ExecutionResponse, BackendError> {
        let plan = self.planner.plan(user_message).await?;
        let mut tool_results = Vec::new();

        for tool_call in &plan.tools {
            let result = self.execute_tool(tool_call).await?;
            tool_results.push(result);
        }

        Ok(ExecutionResponse {
            plan,
            results: tool_results,
        })
    }

    pub async fn handle_chat(
        &self,
        session_id: uuid::Uuid,
        user_message: &str,
    ) -> Result<(String, Vec<String>), BackendError> {
        self.sessions
            .update_session(
                session_id,
                ConversationMessage {
                    role: "user".to_string(),
                    content: user_message.to_string(),
                },
            )
            .await?;

        let execution = self.execute(user_message).await?;

        if !execution.plan.tools.is_empty() {
            self.sessions
                .update_session(
                    session_id,
                    ConversationMessage {
                        role: "tool".to_string(),

                        content: format!(
                            "Used tools: {}",
                            execution
                                .plan
                                .tools
                                .iter()
                                .map(|t| format!("{}({})", t.name, t.arguments.query))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                    },
                )
                .await?;
        }

        let answer = self
            .synthesizer
            .synthesize(user_message, &execution)
            .await?;

        self.sessions
            .update_session(
                session_id,
                ConversationMessage {
                    role: "assistant".to_string(),
                    content: answer.clone(),
                },
            )
            .await?;

        let used_tools = execution
            .plan
            .tools
            .iter()
            .map(|tool| tool.name.clone())
            .collect();

        Ok((answer, used_tools))
    }

    async fn execute_tool(
        &self,
        tool_call: &ToolCall,
    ) -> Result<ToolExecutionResult, BackendError> {
        match tool_call.name.as_str() {
            "local_knowledge_search" => {
                let results = self.local_tool.search(&tool_call.arguments.query, 5);
                Ok(ToolExecutionResult {
                    tool_name: tool_call.name.clone(),
                    payload: to_value(results)?,
                })
            }
            "github_search" => {
                let result = self
                    .github_tool
                    .search(&tool_call.arguments.query, 5)
                    .await?;
                Ok(ToolExecutionResult {
                    tool_name: tool_call.name.clone(),
                    payload: to_value(result)?,
                })
            }
            _ => {
                tracing::warn!("unknown tool requested by planner: {}", tool_call.name);
                Ok(ToolExecutionResult {
                    tool_name: tool_call.name.clone(),
                    payload: serde_json::json!({
                        "error": "unknown tool"
                    }),
                })
            }
        }
    }
}
