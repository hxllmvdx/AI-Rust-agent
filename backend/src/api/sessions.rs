use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::{
    error::BackendError,
    models::sessions::{CreateSessionResponse, HistoryResponse, ResetSessionResponse},
    state::AppState,
};

pub async fn create_session_handler(
    State(state): State<AppState>,
) -> Result<Json<CreateSessionResponse>, StatusCode> {
    let session_id = state
        .sessions
        .create_session()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CreateSessionResponse { session_id }))
}

pub async fn get_history_handler(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<HistoryResponse>, StatusCode> {
    let session = state
        .sessions
        .get_session(session_id)
        .await
        .map_err(|err| match err {
            BackendError::SessionNotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    Ok(Json(HistoryResponse {
        session_id: session.session_id,
        messages: session.messages,
    }))
}

pub async fn reset_session_handler(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> Result<Json<ResetSessionResponse>, StatusCode> {
    state
        .sessions
        .reset_session(session_id)
        .await
        .map_err(|err| match err {
            BackendError::SessionNotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?;

    Ok(Json(ResetSessionResponse { status: "ok" }))
}
