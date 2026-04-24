use serde_json::to_value;

use crate::{
    agent::policy,
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
        let raw_plan = if let Some(plan) = policy::fast_path_plan(user_message) {
            tracing::info!("using fast-path planner for request");
            plan
        } else {
            let planner_started = std::time::Instant::now();
            let raw_plan = self.planner.plan(user_message).await?;
            tracing::info!("raw planner output: {:?}", raw_plan);
            tracing::info!("planner took {:?}", planner_started.elapsed());
            raw_plan
        };

        let policy_started = std::time::Instant::now();
        let plan = policy::apply_tool_policy(user_message, raw_plan);
        tracing::info!("filtered planner output: {:?}", plan);
        tracing::info!("policy filtering took {:?}", policy_started.elapsed());

        let tool_results = self.execute_tools(&plan).await?;

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
        let chat_started = std::time::Instant::now();

        self.sessions
            .append_message(
                session_id,
                ConversationMessage {
                    role: "user".to_string(),
                    content: user_message.to_string(),
                },
            )
            .await?;

        if let Some(answer) = policy::fast_path_response(user_message) {
            self.sessions
                .append_message(
                    session_id,
                    ConversationMessage {
                        role: "assistant".to_string(),
                        content: answer.clone(),
                    },
                )
                .await?;

            tracing::info!("handle_chat took {:?}", chat_started.elapsed());
            return Ok((answer, Vec::new()));
        }

        let execution = self.execute(user_message).await?;
        let session = self.sessions.get_session(session_id).await?;

        if !execution.plan.tools.is_empty() {
            self.sessions
                .append_message(
                    session_id,
                    ConversationMessage {
                        role: "tool".to_string(),

                        content: describe_tool_activity(&execution),
                    },
                )
                .await?;
        }

        let synth_started = std::time::Instant::now();
        let answer = self
            .synthesizer
            .synthesize(user_message, &session.messages, &execution)
            .await?;
        tracing::info!("synthesizer took {:?}", synth_started.elapsed());

        self.sessions
            .append_message(
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

        tracing::info!("handle_chat took {:?}", chat_started.elapsed());

        Ok((answer, used_tools))
    }

    async fn execute_tools(
        &self,
        plan: &crate::models::tool::ToolPlan,
    ) -> Result<Vec<ToolExecutionResult>, BackendError> {
        match plan.tools.as_slice() {
            [] => Ok(Vec::new()),
            [tool] => {
                let tool_started = std::time::Instant::now();
                let result = self.execute_tool(tool).await;
                tracing::info!(
                    "tool {} took {:?} (success={})",
                    tool.name,
                    tool_started.elapsed(),
                    result.success
                );
                Ok(vec![result])
            }
            [first, second] => {
                let started = std::time::Instant::now();
                let first_name = first.name.clone();
                let second_name = second.name.clone();
                let (first_result, second_result) =
                    tokio::join!(self.execute_tool(first), self.execute_tool(second));
                tracing::info!(
                    "tools {} and {} took {:?} (successes: {}, {})",
                    first_name,
                    second_name,
                    started.elapsed(),
                    first_result.success,
                    second_result.success
                );
                Ok(vec![first_result, second_result])
            }
            _ => {
                let mut tool_results = Vec::with_capacity(plan.tools.len());
                for tool_call in &plan.tools {
                    let tool_started = std::time::Instant::now();
                    let result = self.execute_tool(tool_call).await;
                    tracing::info!(
                        "tool {} took {:?} (success={})",
                        tool_call.name,
                        tool_started.elapsed(),
                        result.success
                    );
                    tool_results.push(result);
                }
                Ok(tool_results)
            }
        }
    }

    async fn execute_tool(&self, tool_call: &ToolCall) -> ToolExecutionResult {
        match tool_call.name.as_str() {
            "local_knowledge_search" => {
                let results = self.local_tool.search(&tool_call.arguments.query, 5);
                match to_value(results) {
                    Ok(payload) => ToolExecutionResult::success(tool_call.name.clone(), payload),
                    Err(err) => {
                        tracing::warn!("tool {} failed: {:?}", tool_call.name, err);
                        ToolExecutionResult::failure(
                            tool_call.name.clone(),
                            format!("serialization failed: {err}"),
                        )
                    }
                }
            }
            "github_search" => match self.github_tool.search(&tool_call.arguments.query, 5).await {
                Ok(result) => match to_value(result) {
                    Ok(payload) => ToolExecutionResult::success(tool_call.name.clone(), payload),
                    Err(err) => {
                        tracing::warn!("tool {} failed: {:?}", tool_call.name, err);
                        ToolExecutionResult::failure(
                            tool_call.name.clone(),
                            format!("serialization failed: {err}"),
                        )
                    }
                },
                Err(err) => {
                    tracing::warn!("tool {} failed: {:?}", tool_call.name, err);
                    ToolExecutionResult::failure(tool_call.name.clone(), err.to_string())
                }
            },
            _ => {
                tracing::warn!("unknown tool requested by planner: {}", tool_call.name);
                ToolExecutionResult::failure(tool_call.name.clone(), "unknown tool".to_string())
            }
        }
    }
}

fn describe_tool_activity(execution: &ExecutionResponse) -> String {
    let activity = execution
        .plan
        .tools
        .iter()
        .map(|tool| {
            let status = execution
                .results
                .iter()
                .find(|result| result.tool_name == tool.name)
                .map(|result| if result.success { "ok" } else { "failed" })
                .unwrap_or("unknown");
            format!("{}({}) [{status}]", tool.name, tool.arguments.query)
        })
        .collect::<Vec<_>>()
        .join(", ");

    format!("Used tools: {activity}")
}
