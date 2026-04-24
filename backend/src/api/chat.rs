use axum::{Json, extract::State, http::StatusCode};

use crate::{
    models::chat::{ChatRequest, ChatResponse},
    state::AppState,
};

pub async fn chat_handler(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, StatusCode> {
    let (answer, used_tools) = state
        .orchestrator
        .handle_chat(req.session_id, &req.message)
        .await
        .map_err(|err| {
            tracing::error!("chat_handler failed: {err:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(ChatResponse { answer, used_tools }))
}
