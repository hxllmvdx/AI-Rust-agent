use thiserror::Error;

#[derive(Error, Debug)]
pub enum BackendError {
    #[error("error loading config from .env")]
    ConfigError(#[from] envy::Error),

    #[error("redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("json serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("session not found")]
    SessionNotFound,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("http client error: {0}")]
    ReqwestError(#[from] reqwest::Error),
}
