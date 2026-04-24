use serde::Serialize;
use serde_json::Value;

use crate::models::tool::ToolPlan;

#[derive(Debug, Clone, Serialize)]
pub struct ToolExecutionResult {
    pub tool_name: String,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionResponse {
    pub plan: ToolPlan,
    pub results: Vec<ToolExecutionResult>,
}
