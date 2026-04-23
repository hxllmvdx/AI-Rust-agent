use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct DebugLLMRequest {
    pub prompt: String,
}

#[derive(Debug, Serialize)]
pub struct DebugLLMResponse {
    pub answer: String,
}

pub async fn debug_llm_handler(
    State(state): State<AppState>,
    Json(req): Json<DebugLLMRequest>,
) -> Result<Json<DebugLLMResponse>, StatusCode> {
    let answer = state
        .llm
        .simple_user_prompt(&req.prompt)
        .await
        .map_err(|err| {
            tracing::error!("debug_llm_handler failed: {err:?}");

            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(DebugLLMResponse { answer }))
}
