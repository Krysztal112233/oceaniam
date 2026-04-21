use std::sync::Arc;

use im::Vector;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct Jwk {
    pub kty: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub kid: Option<String>,

    #[serde(rename = "use", skip_serializing_if = "Option::is_none")]
    pub use_: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub alg: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub e: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct JwkSet {
    pub keys: Vector<Jwk>,
}

/// NOTE: THIS STRUCT JUST FOR SIMPLE SCHEMA. DONT CONSTRUCT OR USE IT.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct JwkSetSchema {
    pub keys: Vec<Jwk>,
}

// TODO: OPTIMIZE here
impl From<JwkSet> for jsonwebtoken::jwk::JwkSet {
    fn from(value: JwkSet) -> Self {
        let value = serde_json::to_value(value).unwrap();

        serde_json::from_value(value).unwrap()
    }
}

// TODO: OPTIMIZE here
impl From<Jwk> for jsonwebtoken::jwk::Jwk {
    fn from(value: Jwk) -> Self {
        let value = serde_json::to_value(value).unwrap();

        serde_json::from_value(value).unwrap()
    }
}

// TODO: OPTIMIZE here
impl From<jsonwebtoken::jwk::Jwk> for Jwk {
    fn from(value: jsonwebtoken::jwk::Jwk) -> Self {
        let value = serde_json::to_value(value).unwrap();

        serde_json::from_value(value).unwrap()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ManagedJwkSet {
    jwks: Arc<RwLock<JwkSet>>,
}

impl ManagedJwkSet {
    pub fn new(jwks: JwkSet) -> Self {
        Self {
            jwks: Arc::new(RwLock::new(jwks)),
        }
    }

    pub fn jwks(&self) -> JwkSet {
        self.jwks.read().clone()
    }

    pub fn set_jwks(&self, jwks: JwkSet) {
        *self.jwks.write() = jwks;
    }
}

impl From<JwkSet> for ManagedJwkSet {
    fn from(value: JwkSet) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{Jwk, JwkSet, ManagedJwkSet};
    use im::vector;

    // NOTE: AI-generated test
    #[test]
    fn managed_jwk_set_can_be_constructed_from_jwk_set() {
        let jwks = JwkSet {
            keys: vector![Jwk {
                kty: "RSA".to_string(),
                kid: Some("key-1".to_string()),
                use_: Some("sig".to_string()),
                alg: Some("PS512".to_string()),
                n: Some("modulus".to_string()),
                e: Some("AQAB".to_string()),
            }],
        };

        let managed = ManagedJwkSet::new(jwks.clone());
        let keys = managed.jwks().keys;

        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].kid.as_deref(), Some("key-1"));
        assert_eq!(keys[0].alg.as_deref(), Some("PS512"));
    }

    // NOTE: AI-generated test
    #[test]
    fn managed_jwk_set_can_be_updated_directly() {
        let managed = ManagedJwkSet::default();
        let jwks = JwkSet {
            keys: vector![Jwk {
                kty: "RSA".to_string(),
                kid: Some("key-2".to_string()),
                use_: Some("sig".to_string()),
                alg: Some("PS256".to_string()),
                n: Some("next-modulus".to_string()),
                e: Some("AQAB".to_string()),
            }],
        };

        managed.set_jwks(jwks.clone());
        let keys = managed.jwks().keys;

        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].kid.as_deref(), Some("key-2"));
        assert_eq!(keys[0].alg.as_deref(), Some("PS256"));
    }
}
