use std::sync::Arc;

use im::Vector;
use oceaniam_crypto::{DecodingKey, ProviderJwk, ProviderJwkSet, decoding_key_from_jwk};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::Error;

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

impl From<JwkSet> for ProviderJwkSet {
    fn from(value: JwkSet) -> Self {
        serde_json::from_value(
            serde_json::to_value(value).expect("serialize internal JWK set representation"),
        )
        .expect("convert valid internal JWK set representation")
    }
}

impl From<Jwk> for ProviderJwk {
    fn from(value: Jwk) -> Self {
        serde_json::from_value(
            serde_json::to_value(value).expect("serialize internal JWK representation"),
        )
        .expect("convert valid internal JWK representation")
    }
}

impl From<ProviderJwk> for Jwk {
    fn from(value: ProviderJwk) -> Self {
        serde_json::from_value(
            serde_json::to_value(value).expect("serialize provider JWK representation"),
        )
        .expect("convert supported provider JWK representation")
    }
}

impl JwkSet {
    #[tracing::instrument(
        level = "info",
        name = "auth.jwk.decoding_key",
        skip_all,
        fields(otel.kind = "internal", key.id = %kid)
    )]
    pub fn decoding_key_for_kid(&self, kid: &str) -> Result<DecodingKey, Error> {
        let jwk = self
            .keys
            .iter()
            .find(|jwk| jwk.kid.as_deref() == Some(kid))
            .ok_or_else(|| Error::KidNotFound {
                kid: kid.to_string(),
                location: snafu::location!(),
            })?;
        let jwk: ProviderJwk =
            serde_json::from_value(serde_json::to_value(jwk).map_err(|source| {
                Error::Internal {
                    msg: format!("failed to serialize JWK: {source}"),
                    location: snafu::location!(),
                }
            })?)
            .map_err(|source| Error::Internal {
                msg: format!("invalid JWK: {source}"),
                location: snafu::location!(),
            })?;

        Ok(decoding_key_from_jwk(&jwk)?)
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
    use crate::error::Error;
    use im::vector;
    use oceaniam_crypto::ProviderJwkSet;

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
    fn jwk_lookup_rejects_unknown_kids_and_malformed_rsa_components() {
        let jwks = JwkSet {
            keys: vector![Jwk {
                kty: "RSA".to_owned(),
                kid: Some("known-key".to_owned()),
                use_: None,
                alg: Some("RS256".to_owned()),
                n: Some("***".to_owned()),
                e: Some("AQAB".to_owned()),
            }],
        };

        assert!(matches!(
            jwks.decoding_key_for_kid("unknown-key"),
            Err(Error::KidNotFound { .. })
        ));
        assert!(jwks.decoding_key_for_kid("known-key").is_err());
    }

    // NOTE: AI-generated test
    #[test]
    fn provider_jwk_compatibility_conversions_preserve_rsa_fields() {
        let internal = JwkSet {
            keys: vector![Jwk {
                kty: "RSA".to_owned(),
                kid: Some("key-compat".to_owned()),
                use_: Some("sig".to_owned()),
                alg: Some("RS256".to_owned()),
                n: Some("AQAB".to_owned()),
                e: Some("AQAB".to_owned()),
            }],
        };

        let provider = ProviderJwkSet::from(internal);
        let round_trip = Jwk::from(provider.keys[0].clone());
        assert_eq!(round_trip.kty, "RSA");
        assert_eq!(round_trip.kid.as_deref(), Some("key-compat"));
        assert_eq!(round_trip.use_.as_deref(), Some("sig"));
        assert_eq!(round_trip.alg.as_deref(), Some("RS256"));
        assert_eq!(round_trip.n.as_deref(), Some("AQAB"));
        assert_eq!(round_trip.e.as_deref(), Some("AQAB"));
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
