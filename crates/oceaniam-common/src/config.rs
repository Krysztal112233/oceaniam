use std::time::Duration;

use config::Config;
use sea_orm::ConnectOptions;
use serde::{Deserialize, Serialize};
use tap::Pipe;

use crate::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub dsn: String,

    pub slow_statements_logging_threshold: Option<u64>,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
}

impl From<DatabaseConfig> for ConnectOptions {
    fn from(
        DatabaseConfig {
            dsn,
            slow_statements_logging_threshold,
            max_connections,
            min_connections,
        }: DatabaseConfig,
    ) -> Self {
        ConnectOptions::new(dsn)
            .pipe_borrow_mut(|it| match slow_statements_logging_threshold {
                Some(milis) => it.sqlx_slow_statements_logging_settings(
                    log::LevelFilter::Warn,
                    Duration::from_micros(milis),
                ),
                _ => it,
            })
            .pipe_borrow_mut(|it| match max_connections {
                Some(c) => it.max_connections(c),
                _ => it,
            })
            .pipe_borrow_mut(|it| match min_connections {
                Some(c) => it.min_connections(c),
                _ => it,
            })
            .pipe_borrow_mut(|it| it.sqlx_logging(false))
            .to_owned()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub addr: String,
    pub database: DatabaseConfig,
}

impl BackendConfig {
    pub fn new() -> Result<Self, Error> {
        Ok(Config::builder()
            .add_source(config::File::with_name("config.toml"))
            .add_source(config::Environment::with_prefix("OCEANIAM"))
            .build()?
            .try_deserialize()?)
    }
}
