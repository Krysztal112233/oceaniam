use std::collections::HashMap;

use config::Config;
use serde::{Deserialize, Deserializer, Serialize};
use snafu::Snafu;
use url::{Host, Url};

use oceaniam_application_secret::ApplicationSecretKeyring;

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicBaseUrl(Url);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Snafu)]
pub enum PublicBaseUrlError {
    #[snafu(display("public base URL must use the `http` or `https` scheme"))]
    UnsupportedScheme,

    #[snafu(display("public base URL must include a host"))]
    MissingHost,

    #[snafu(display("public base URL must not include credentials"))]
    CredentialsNotAllowed,

    #[snafu(display("public base URL must not include a non-root path"))]
    NonRootPathNotAllowed,

    #[snafu(display("public base URL must not include a query"))]
    QueryNotAllowed,

    #[snafu(display("public base URL must not include a fragment"))]
    FragmentNotAllowed,
}

impl PublicBaseUrl {
    pub fn as_url(&self) -> &Url {
        &self.0
    }

    /// Whether this trusted, canonical origin is safe for the authorization-entry preview.
    ///
    /// HTTP is intentionally limited to the three fixed loopback hosts. This operates on the
    /// canonical `Url` representation used by configuration; registered redirect URIs retain
    /// their separate raw-string policy.
    pub fn allows_oidc_authorization_entry_preview(&self) -> bool {
        if self.0.scheme() == "https" {
            return true;
        }
        if self.0.scheme() != "http" {
            return false;
        }

        match self.0.host() {
            Some(Host::Domain("localhost")) => true,
            Some(Host::Ipv4(address)) => address == std::net::Ipv4Addr::LOCALHOST,
            Some(Host::Ipv6(address)) => address == std::net::Ipv6Addr::LOCALHOST,
            _ => false,
        }
    }
}

impl TryFrom<Url> for PublicBaseUrl {
    type Error = PublicBaseUrlError;

    fn try_from(value: Url) -> Result<Self, Self::Error> {
        if !matches!(value.scheme(), "http" | "https") {
            return Err(PublicBaseUrlError::UnsupportedScheme);
        }
        if value.host().is_none() {
            return Err(PublicBaseUrlError::MissingHost);
        }
        if !value.username().is_empty() || value.password().is_some() {
            return Err(PublicBaseUrlError::CredentialsNotAllowed);
        }
        if value.path() != "/" {
            return Err(PublicBaseUrlError::NonRootPathNotAllowed);
        }
        if value.query().is_some() {
            return Err(PublicBaseUrlError::QueryNotAllowed);
        }
        if value.fragment().is_some() {
            return Err(PublicBaseUrlError::FragmentNotAllowed);
        }

        Ok(Self(value))
    }
}

impl<'de> Deserialize<'de> for PublicBaseUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Url::deserialize(deserializer)?
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}

pub const DEFAULT_SLOW_STATEMENTS_LOGGING_THRESHOLD: u64 = 200;
pub const DEFAULT_DATABASE_MAX_CONNECTIONS: u32 = 128;
pub const DEFAULT_DATABASE_MIN_CONNECTIONS: u32 = 4;

const fn default_slow_statements_logging_threshold() -> Option<u64> {
    Some(DEFAULT_SLOW_STATEMENTS_LOGGING_THRESHOLD)
}

const fn default_database_max_connections() -> Option<u32> {
    Some(DEFAULT_DATABASE_MAX_CONNECTIONS)
}

const fn default_database_min_connections() -> Option<u32> {
    Some(DEFAULT_DATABASE_MIN_CONNECTIONS)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub dsn: String,

    #[serde(default = "default_slow_statements_logging_threshold")]
    pub slow_statements_logging_threshold: Option<u64>,
    #[serde(default = "default_database_max_connections")]
    pub max_connections: Option<u32>,
    #[serde(default = "default_database_min_connections")]
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

/// Default-disabled OIDC protocol previews.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OidcConfig {
    #[serde(default)]
    pub authorization_entry_preview_enabled: bool,
}

pub const DEFAULT_ADDR: &str = "0.0.0.0:8000";

fn default_addr() -> String {
    DEFAULT_ADDR.to_owned()
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
    #[serde(default = "default_addr")]
    pub addr: String,
    pub public_base_url: PublicBaseUrl,
    pub database: DatabaseConfig,
    pub cors: CorsConfig,

    #[serde(default)]
    pub cookie: CookieConfig,

    #[serde(default)]
    pub oidc: OidcConfig,

    #[serde(default)]
    pub telemetry: TelemetryConfig,

    pub master_key: String,

    #[serde(default)]
    pub application_secret_hmac: Option<ApplicationSecretKeyring>,
}

impl BackendConfig {
    /// Loads configuration from `OCEANIAM_` environment variables.
    ///
    /// Environment variables are the single configuration source; the serde
    /// defaults in this module provide the fallback values. Nested keys use
    /// `__` (for example `OCEANIAM_DATABASE__DSN` maps to `database.dsn`).
    /// The repository-root `.env.example` documents every key and is kept in
    /// sync with this schema by the drift tests below.
    pub fn new() -> Result<Self, Error> {
        Ok(Config::builder()
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

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: AI-generated test
    #[test]
    fn public_base_url_accepts_http_and_https_origins() {
        for value in ["http://localhost:8000", "https://iam.example.com"] {
            let base_url = serde_json::from_str::<PublicBaseUrl>(&format!("\"{value}\""))
                .expect("valid public base URL");

            assert_eq!(base_url.as_url().path(), "/");
        }
    }

    // NOTE: AI-generated test
    #[test]
    fn public_base_url_rejects_ambiguous_or_unsafe_components() {
        for value in [
            "ftp://iam.example.com",
            "https://user:password@iam.example.com",
            "https://iam.example.com/nested",
            "https://iam.example.com?region=test",
            "https://iam.example.com#fragment",
        ] {
            assert!(
                serde_json::from_str::<PublicBaseUrl>(&format!("\"{value}\"")).is_err(),
                "unexpectedly accepted {value}"
            );
        }
    }

    // NOTE: AI-generated test
    #[test]
    fn authorization_preview_accepts_only_https_or_canonical_fixed_loopback_origins() {
        for value in [
            "https://iam.example.com",
            "http://localhost:8000",
            "http://127.0.0.1:8000",
            "http://[::1]:8000",
            // `url` intentionally canonicalizes this trusted configuration alias.
            "http://127.1:8000",
        ] {
            let base_url = serde_json::from_str::<PublicBaseUrl>(&format!("\"{value}\""))
                .expect("valid public base URL");
            assert!(
                base_url.allows_oidc_authorization_entry_preview(),
                "unexpectedly rejected {value}"
            );
        }

        for value in [
            "http://iam.example.com",
            "http://sub.localhost",
            "http://localhost.example",
            "http://127.0.0.2",
            "http://[::2]",
            "http://[::ffff:127.0.0.1]",
        ] {
            let base_url = serde_json::from_str::<PublicBaseUrl>(&format!("\"{value}\""))
                .expect("valid public base URL");
            assert!(
                !base_url.allows_oidc_authorization_entry_preview(),
                "unexpectedly accepted {value}"
            );
        }
    }

    // NOTE: AI-generated test
    #[test]
    fn oidc_authorization_entry_preview_defaults_to_disabled() {
        let config: OidcConfig = serde_json::from_str("{}").expect("OIDC config should parse");
        assert!(!config.authorization_entry_preview_enabled);
    }

    // Drift guards: keep the repository-root `.env.example` in sync with the
    // configuration schema in this module. When you add, rename, or remove a
    // configuration field, update `.env.example` and these lists in the same
    // commit — the tests below fail otherwise.

    /// Every static configuration key on the schema side.
    const KNOWN_ENV_KEYS: &[&str] = &[
        "OCEANIAM_ADDR",
        "OCEANIAM_APPLICATION_SECRET_HMAC__CURRENT_VERSION",
        "OCEANIAM_COOKIE__SECURE",
        "OCEANIAM_CORS__ALLOW_ORIGIN",
        "OCEANIAM_DATABASE__DSN",
        "OCEANIAM_DATABASE__MAX_CONNECTIONS",
        "OCEANIAM_DATABASE__MIN_CONNECTIONS",
        "OCEANIAM_DATABASE__SLOW_STATEMENTS_LOGGING_THRESHOLD",
        "OCEANIAM_MASTER_KEY",
        "OCEANIAM_OIDC__AUTHORIZATION_ENTRY_PREVIEW_ENABLED",
        "OCEANIAM_PUBLIC_BASE_URL",
        "OCEANIAM_TELEMETRY__ENABLED",
        "OCEANIAM_TELEMETRY__OTLP_ENDPOINT",
        "OCEANIAM_TELEMETRY__OTLP_TRACES_ENDPOINT",
        "OCEANIAM_TELEMETRY__SERVICE_NAME",
        "OCEANIAM_TELEMETRY__TRACE_SAMPLE_RATIO",
    ];

    /// Dynamic key families (versioned keyrings and header maps).
    const DYNAMIC_KEY_PREFIXES: &[&str] = &[
        "OCEANIAM_APPLICATION_SECRET_HMAC__KEYS__",
        "OCEANIAM_TELEMETRY__OTLP_HEADERS__",
    ];

    /// Keys without built-in defaults: the example must set them actively.
    const REQUIRED_ENV_KEYS: &[&str] = &[
        "OCEANIAM_CORS__ALLOW_ORIGIN",
        "OCEANIAM_DATABASE__DSN",
        "OCEANIAM_MASTER_KEY",
        "OCEANIAM_PUBLIC_BASE_URL",
    ];

    /// Parses the repository-root `.env.example` into `(key, value, commented)`.
    fn env_example_entries() -> Vec<(String, String, bool)> {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../.env.example");
        let content = std::fs::read_to_string(path).expect("read repository-root .env.example");

        content
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() {
                    return None;
                }
                let commented = line.starts_with('#');
                let line = line.trim_start_matches('#').trim();
                let (key, value) = line.split_once('=')?;
                if !key.starts_with("OCEANIAM_") {
                    return None;
                }
                let value = value.trim();
                let value = value
                    .strip_prefix('"')
                    .and_then(|it| it.strip_suffix('"'))
                    .unwrap_or(value)
                    .to_owned();
                Some((key.to_owned(), value, commented))
            })
            .collect()
    }

    #[test]
    fn env_example_contains_no_unknown_keys() {
        for (key, _, _) in env_example_entries() {
            let known = KNOWN_ENV_KEYS.contains(&key.as_str())
                || DYNAMIC_KEY_PREFIXES
                    .iter()
                    .any(|prefix| key.starts_with(prefix));
            assert!(
                known,
                "unknown key `{key}` in .env.example; align the example with the \
                 configuration schema (or update the drift lists for intentional changes)"
            );
        }
    }

    #[test]
    fn env_example_documents_every_schema_key() {
        let documented: std::collections::HashSet<String> = env_example_entries()
            .into_iter()
            .map(|(key, _, _)| key)
            .collect();

        for key in KNOWN_ENV_KEYS {
            assert!(
                documented.contains(*key),
                "config key `{key}` missing from .env.example; document it there \
                 (commented is acceptable for optional keys)"
            );
        }
    }

    #[test]
    fn env_example_actively_sets_required_keys() {
        let entries = env_example_entries();

        for required in REQUIRED_ENV_KEYS {
            let entry = entries.iter().find(|(key, _, _)| key == required);
            assert!(
                matches!(entry, Some((_, _, false))),
                "required key `{required}` must be set (not commented) in .env.example"
            );
        }
    }

    #[test]
    fn env_example_values_deserialize_into_backend_config() {
        let mut body = String::new();

        for (key, value, commented) in env_example_entries() {
            if commented {
                continue;
            }

            // The example ships an empty placeholder for the HMAC keyring; the
            // keyring deserializer validates eagerly, so substitute a valid key.
            let value = if key.starts_with("OCEANIAM_APPLICATION_SECRET_HMAC__KEYS__")
                && value.is_empty()
            {
                "ab".repeat(32)
            } else {
                value
            };

            let rendered = if value.parse::<bool>().is_ok() || value.parse::<f64>().is_ok() {
                value
            } else {
                format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
            };

            let path = key
                .trim_start_matches("OCEANIAM_")
                .to_lowercase()
                .replace("__", ".");
            body.push_str(&format!("{path} = {rendered}\n"));
        }

        Config::builder()
            .add_source(config::File::from_str(&body, config::FileFormat::Toml))
            .build()
            .expect("build config from .env.example values")
            .try_deserialize::<BackendConfig>()
            .expect(".env.example values must deserialize into BackendConfig");
    }
}
