use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::{models::execution::ExecutionResponse, state::AppState};

#[derive(Debug, Deserialize)]
pub struct DebugExecuteRequest {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct DebugExecuteResponse {
    pub execution: ExecutionResponse,
}

pub async fn debug_execute_handler(
    State(state): State<AppState>,
    Json(req): Json<DebugExecuteRequest>,
) -> Result<Json<DebugExecuteResponse>, StatusCode> {
    let execution = state
        .orchestrator
        .execute(&req.message)
        .await
        .map_err(|err| {
            tracing::error!("debug_execute_handler failed: {err:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(DebugExecuteResponse { execution }))
}
