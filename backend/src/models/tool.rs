use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolArguments {
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: ToolArguments,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPlan {
    pub need_tools: bool,
    pub tools: Vec<ToolCall>,
}
