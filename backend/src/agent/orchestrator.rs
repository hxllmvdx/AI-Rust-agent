use serde_json::to_value;

use crate::{
    error::BackendError,
    models::{
        execution::{ExecutionResponse, ToolExecutionResult},
        tool::ToolCall,
    },
    tools::{github::GitHubTool, local_data::LocalKnowledgeTool},
};

use super::planner::PlannerService;

#[derive(Clone)]
pub struct OrchestratorService {
    planner: PlannerService,
    local_tool: LocalKnowledgeTool,
    github_tool: GitHubTool,
}

impl OrchestratorService {
    pub fn new(
        planner: PlannerService,
        local_tool: LocalKnowledgeTool,
        github_tool: GitHubTool,
    ) -> Self {
        Self {
            planner,
            local_tool,
            github_tool,
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
