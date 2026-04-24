use crate::agent::{orchestrator::OrchestratorService, planner::PlannerService};
use crate::services::llm::LlmService;
use crate::services::session_store::SessionStore;
use crate::tools::{github::GitHubTool, local_data::LocalKnowledgeTool};

#[derive(Clone)]
pub struct AppState {
    pub app_name: String,
    pub sessions: SessionStore,
    pub llm: LlmService,
    pub planner: PlannerService,
    pub local_tool: LocalKnowledgeTool,
    pub github_tool: GitHubTool,
    pub orchestrator: OrchestratorService,
}
