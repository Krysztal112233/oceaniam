use oceaniam_common::consts;
use serde::{Deserialize, Serialize};

/// Absolute upper bound for any development-account TTL, matching the pgmq delay
/// parameter (PostgreSQL `integer`).
pub const MAX_DEV_ACCOUNT_TTL_SECONDS: u64 = 2_147_483_647;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApplicationConfiguration {
    pub auth: AuthConfiguration,
    pub registration: RegistrationConfiguration,
    #[serde(default)]
    pub development_accounts: DevelopmentAccountsConfiguration,
    #[serde(default)]
    pub oidc: OidcConfiguration,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfiguration {
    pub token: TokenConfiguration,
    pub password: PasswordConfiguration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenConfiguration {
    pub issuer: String,
    pub audience: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PasswordConfiguration {
    pub argon2: Argon2Configuration,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RegistrationConfiguration {
    pub enabled: bool,
}

/// Per-application policy for development accounts.
///
/// `enabled=false` rejects development-account creation entirely (fail-closed).
/// `default_ttl_seconds` applies when the request omits `ttl_seconds`;
/// `max_ttl_seconds` caps any requested TTL.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DevelopmentAccountsConfiguration {
    pub enabled: bool,
    pub default_ttl_seconds: u64,
    pub max_ttl_seconds: u64,
}

/// Per-application policy for OIDC Provider behavior.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct OidcConfiguration {
    /// Allows explicitly registered HTTP redirect URIs on loopback hosts for development.
    pub allow_insecure_loopback_redirect_uris: bool,
}

impl Default for DevelopmentAccountsConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            default_ttl_seconds: 3600,
            max_ttl_seconds: 86400,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Argon2Configuration {
    pub m_cost: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

impl Default for Argon2Configuration {
    fn default() -> Self {
        Self {
            m_cost: 12288,
            t_cost: 3,
            p_cost: 1,
        }
    }
}

impl Default for TokenConfiguration {
    fn default() -> Self {
        Self {
            issuer: consts::DEFAULT_JWT_ISSUER.to_owned(),
            audience: vec![consts::DEFAULT_JWT_AUDIENCE.to_owned()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: AI-generated test
    #[test]
    fn legacy_application_configuration_defaults_to_secure_oidc_redirects() {
        let mut value = serde_json::to_value(ApplicationConfiguration::default())
            .expect("default application configuration should serialize");
        value
            .as_object_mut()
            .expect("application configuration should be an object")
            .remove("oidc");

        let configuration: ApplicationConfiguration = serde_json::from_value(value)
            .expect("legacy application configuration should deserialize");

        assert!(!configuration.oidc.allow_insecure_loopback_redirect_uris);
    }
}
