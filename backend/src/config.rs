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

fn default_ollama_url() -> String {
    "http://ollama:11434".to_string()
}

fn default_ollama_keep_alive() -> String {
    "10m".to_string()
}

fn default_ollama_planner_model() -> String {
    "qwen3:8b".to_string()
}

fn default_ollama_synthesizer_model() -> String {
    "qwen3:8b".to_string()
}

fn default_ollama_planner_thinking() -> bool {
    false
}

fn default_ollama_synthesizer_thinking() -> bool {
    true
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

    #[serde(default = "default_ollama_url", alias = "OLLAMA_BASE_URL")]
    pub ollama_url: String,

    #[serde(default = "default_ollama_planner_model")]
    pub ollama_planner_model: String,

    #[serde(default = "default_ollama_synthesizer_model")]
    pub ollama_synthesizer_model: String,

    #[serde(default = "default_ollama_keep_alive")]
    pub ollama_keep_alive: String,

    #[serde(default = "default_ollama_planner_thinking")]
    pub ollama_planner_thinking: bool,

    #[serde(default = "default_ollama_synthesizer_thinking")]
    pub ollama_synthesizer_thinking: bool,

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
