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
    "qwen3:4b".to_string()
}

fn default_ollama_synthesizer_model() -> String {
    "qwen3:8b".to_string()
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

    pub github_token: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, BackendError> {
        dotenvy::dotenv().ok();
        envy::from_env().map_err(|e| BackendError::ConfigError(e))
    }
}
