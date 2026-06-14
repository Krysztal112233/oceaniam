use oceaniam_common::consts;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApplicationConfiguration {
    pub auth: AuthConfiguration,
    pub registration: RegistrationConfiguration,
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
