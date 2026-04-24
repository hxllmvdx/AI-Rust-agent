use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::{models::github::GitHubSearchResult, state::AppState};

#[derive(Debug, Deserialize)]
pub struct DebugGitHubSearchRequest {
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct DebugGitHubSearchResponse {
    pub results: Vec<GitHubSearchResult>,
}

pub async fn debug_github_search_handler(
    State(state): State<AppState>,
    Json(req): Json<DebugGitHubSearchRequest>,
) -> Result<Json<DebugGitHubSearchResponse>, StatusCode> {
    let results = state
        .github_tool
        .search(&req.query, 5)
        .await
        .map_err(|err| {
            tracing::error!("debug_github_search_handler failed: {err:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(DebugGitHubSearchResponse { results }))
}
