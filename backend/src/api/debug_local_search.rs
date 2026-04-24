use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::{models::local_data::KnowledgeSearchResult, state::AppState};

#[derive(Debug, Deserialize)]
pub struct DebugLocalSearchRequest {
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct DebugLocalSearchResponse {
    pub results: Vec<KnowledgeSearchResult>,
}

pub async fn debug_local_search_handler(
    State(state): State<AppState>,
    Json(req): Json<DebugLocalSearchRequest>,
) -> Result<Json<DebugLocalSearchResponse>, StatusCode> {
    let results = state.local_tool.search(&req.query, 5);

    Ok(Json(DebugLocalSearchResponse { results }))
}
