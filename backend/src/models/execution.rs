use serde::Serialize;
use serde_json::Value;

use crate::models::tool::ToolPlan;

#[derive(Debug, Clone, Serialize)]
pub struct ToolExecutionResult {
    pub tool_name: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ToolExecutionResult {
    pub fn success(tool_name: String, payload: Value) -> Self {
        Self {
            tool_name,
            success: true,
            payload: Some(payload),
            error: None,
        }
    }

    pub fn failure(tool_name: String, error: String) -> Self {
        Self {
            tool_name,
            success: false,
            payload: None,
            error: Some(error),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionResponse {
    pub plan: ToolPlan,
    pub results: Vec<ToolExecutionResult>,
}
