use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};

use crate::{models::tool::ToolPlan, state::AppState};

#[derive(Debug, Deserialize)]
pub struct DebugPlanRequest {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct DebugPlanResponse {
    pub plan: ToolPlan,
}

pub async fn debug_plan_handler(
    State(state): State<AppState>,
    Json(req): Json<DebugPlanRequest>,
) -> Result<Json<DebugPlanResponse>, StatusCode> {
    let plan = state.planner.plan(&req.message).await.map_err(|err| {
        tracing::error!("debug_plan_handler failed: {err:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(DebugPlanResponse { plan }))
}
