use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: Uuid,
    pub messages: Vec<ConversationMessage>,
}

#[derive(Debug, Serialize)]
pub struct CreateSessionResponse {
    pub session_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct HistoryResponse {
    pub session_id: Uuid,
    pub messages: Vec<ConversationMessage>,
}

#[derive(Debug, Serialize)]
pub struct ResetSessionResponse {
    pub status: &'static str,
}
