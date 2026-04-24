use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatRequest {
    pub session_id: Uuid,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatResponse {
    pub answer: String,
    pub used_tools: Vec<String>,
}
