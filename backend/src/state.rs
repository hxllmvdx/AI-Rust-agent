use crate::agent::planner::PlannerService;
use crate::services::llm::LlmService;
use crate::services::session_store::SessionStore;

#[derive(Clone)]
pub struct AppState {
    pub app_name: String,
    pub sessions: SessionStore,
    pub llm: LlmService,
    pub planner: PlannerService,
}
