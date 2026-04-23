use thiserror::Error;

#[derive(Error, Debug)]
pub enum BackendError {
    #[error("error loading config from .env")]
    ConfigError(#[from] envy::Error),
}
