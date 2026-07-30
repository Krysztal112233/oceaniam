use std::collections::HashMap;

use config::Config;
use serde::{Deserialize, Serialize};
use url::Url;

use oceaniam_application_secret::ApplicationSecretKeyring;

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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CookieConfig {
    #[serde(default)]
    pub secure: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfiguration {
    pub cron: String,
}

pub const DEFAULT_TELEMETRY_SERVICE_NAME: &str = "oceaniam";
pub const DEFAULT_TRACE_SAMPLE_RATIO: f64 = 1.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryConfig {
    #[serde(default)]
    pub enabled: bool,

    /// Full OTLP/HTTP logs URL, used as-is without appending a path.
    #[serde(default)]
    pub otlp_endpoint: Option<Url>,

    /// Full OTLP/HTTP traces URL, used as-is without appending a path.
    #[serde(default)]
    pub otlp_traces_endpoint: Option<Url>,

    #[serde(default = "default_telemetry_service_name")]
    pub service_name: String,

    /// Head-sampling ratio for traces. `1.0` samples every locally-created root trace.
    #[serde(default = "default_trace_sample_ratio")]
    pub trace_sample_ratio: f64,

    #[serde(default)]
    pub otlp_headers: HashMap<String, String>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            otlp_endpoint: None,
            otlp_traces_endpoint: None,
            service_name: default_telemetry_service_name(),
            trace_sample_ratio: default_trace_sample_ratio(),
            otlp_headers: HashMap::new(),
        }
    }
}

fn default_telemetry_service_name() -> String {
    DEFAULT_TELEMETRY_SERVICE_NAME.to_owned()
}

const fn default_trace_sample_ratio() -> f64 {
    DEFAULT_TRACE_SAMPLE_RATIO
}

#[derive(Debug, Deserialize)]
pub struct BackendConfig {
    pub addr: String,
    pub database: DatabaseConfig,
    pub cors: CorsConfig,

    #[serde(default)]
    pub workers: HashMap<String, WorkerConfiguration>,

    #[serde(default)]
    pub cookie: CookieConfig,

    #[serde(default)]
    pub telemetry: TelemetryConfig,

    pub master_key: String,

    #[serde(default)]
    pub application_secret_hmac: Option<ApplicationSecretKeyring>,
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
