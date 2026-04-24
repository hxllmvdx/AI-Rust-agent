use crate::error::BackendError;
use serde::Deserialize;

fn default_port() -> u16 {
    8080
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_redis_url() -> String {
    "redis://redis:6379/".to_string()
}

fn default_session_ttl() -> u64 {
    60 * 60 * 24
}

fn default_openrouter_base_url() -> String {
    "https://openrouter.ai/api/v1".to_string()
}

fn default_openrouter_planner_model() -> String {
    "openrouter/free".to_string()
}

fn default_openrouter_synthesizer_model() -> String {
    "openrouter/free".to_string()
}

fn default_openrouter_planner_reasoning() -> bool {
    false
}

fn default_openrouter_synthesizer_reasoning() -> bool {
    false
}

fn default_crates_api_base_url() -> String {
    "https://crates.io/api/v1".to_string()
}

fn default_crates_api_user_agent() -> String {
    "ai-rust-agent (local development; set CRATES_API_USER_AGENT for production contact)"
        .to_string()
}

fn default_crates_api_rate_limit_ms() -> u64 {
    1000
}

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_redis_url")]
    pub redis_url: String,

    #[serde(default = "default_session_ttl", alias = "SESSION_TTL_SECS")]
    pub session_ttl: u64,

    #[serde(
        default = "default_openrouter_base_url",
        alias = "OPENROUTER_URL",
        alias = "OLLAMA_BASE_URL"
    )]
    pub openrouter_base_url: String,

    pub openrouter_api_key: String,

    #[serde(
        default = "default_openrouter_planner_model",
        alias = "OLLAMA_PLANNER_MODEL"
    )]
    pub openrouter_planner_model: String,

    #[serde(
        default = "default_openrouter_synthesizer_model",
        alias = "OLLAMA_SYNTHESIZER_MODEL"
    )]
    pub openrouter_synthesizer_model: String,

    #[serde(
        default = "default_openrouter_planner_reasoning",
        alias = "OLLAMA_PLANNER_THINKING"
    )]
    pub openrouter_planner_reasoning: bool,

    #[serde(
        default = "default_openrouter_synthesizer_reasoning",
        alias = "OLLAMA_SYNTHESIZER_THINKING"
    )]
    pub openrouter_synthesizer_reasoning: bool,

    #[serde(default = "default_crates_api_base_url")]
    pub crates_api_base_url: String,

    #[serde(default = "default_crates_api_user_agent")]
    pub crates_api_user_agent: String,

    #[serde(default = "default_crates_api_rate_limit_ms")]
    pub crates_api_rate_limit_ms: u64,

    pub github_token: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, BackendError> {
        dotenvy::dotenv().ok();
        envy::from_env().map_err(|e| BackendError::ConfigError(e))
    }
}
