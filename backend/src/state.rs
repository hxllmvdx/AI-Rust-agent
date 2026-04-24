use crate::agent::{
    orchestrator::OrchestratorService, planner::PlannerService, synthesizer::SynthesizerService,
};
use crate::services::llm::LlmService;
use crate::services::session_store::SessionStore;
use crate::tools::{crates::CratesTool, github::GitHubTool, local_data::LocalKnowledgeTool};

#[derive(Clone)]
pub struct AppState {
    pub app_name: String,
    pub sessions: SessionStore,
    pub llm: LlmService,
    pub planner: PlannerService,
    pub synthesizer: SynthesizerService,
    pub local_tool: LocalKnowledgeTool,
    pub github_tool: GitHubTool,
    pub crates_tool: CratesTool,
    pub orchestrator: OrchestratorService,
}
