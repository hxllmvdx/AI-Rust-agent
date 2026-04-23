use crate::error::BackendError;
use serde::Deserialize;

fn default_port() -> u16 {
    8080
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, BackendError> {
        dotenvy::dotenv().ok();
        envy::from_env().map_err(|e| BackendError::ConfigError(e))
    }
}
