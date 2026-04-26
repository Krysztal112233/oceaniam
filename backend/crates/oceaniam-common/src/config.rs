use std::collections::HashMap;

use config::Config;
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub dsn: String,

    pub slow_statements_logging_threshold: Option<u64>,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub allow_origin: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfiguration {
    pub cron: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub addr: String,
    pub database: DatabaseConfig,
    pub cors: CorsConfig,
    #[serde(default)]
    pub workers: HashMap<String, WorkerConfiguration>,
}

impl BackendConfig {
    pub fn new() -> Result<Self, Error> {
        Ok(Config::builder()
            .add_source(config::File::with_name("config.toml").required(false))
            .add_source(
                config::Environment::with_prefix("OCEANIAM")
                    .prefix_separator("_")
                    .separator("__")
                    .list_separator(","),
            )
            .build()?
            .try_deserialize()?)
    }
}
