use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::{models::crates::CratesSearchResult, state::AppState};

#[derive(Debug, Deserialize)]
pub struct DebugCratesSearchRequest {
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct DebugCratesSearchResponse {
    pub results: Vec<CratesSearchResult>,
}

pub async fn debug_crates_search_handler(
    State(state): State<AppState>,
    Json(req): Json<DebugCratesSearchRequest>,
) -> Result<Json<DebugCratesSearchResponse>, StatusCode> {
    let results = state
        .crates_tool
        .search(&req.query, 5)
        .await
        .map_err(|err| {
            tracing::error!("debug_crates_search_handler failed: {err:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(DebugCratesSearchResponse { results }))
}
