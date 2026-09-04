//! OIDC protocol-boundary JWK Set built on `openidconnect` core types.
//!
//! The internal [`JwkSet`](oceaniam_auth::jwks::JwkSet) stays the source of truth for tenant keyboxes;
//! this module converts it into the official [`CoreJsonWebKeySet`] at the OIDC protocol boundary
//! so the public OIDC surface speaks the same wire format as the `openidconnect` crate.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use openidconnect::core::{CoreJsonWebKey, CoreJsonWebKeySet};
use serde_json::json;
use snafu::{Location, Snafu};

use oceaniam_auth::jwks::JwkSet;

/// Deterministic failures while converting an internal JWK Set into the OIDC core type.
///
/// The conversion is deliberately strict: `openidconnect`'s `JsonWebKeySet` deserialization
/// silently skips keys it cannot parse, so the set is never deserialized wholesale. Each key is
/// validated and converted individually, and any invalid key fails the whole conversion.
#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum OidcJwksError {
    #[snafu(display("JWK #{index} has unsupported key type `{kty}` at {location}"))]
    UnsupportedKeyType {
        index: usize,
        kty: String,
        location: Location,
    },

    #[snafu(display("JWK #{index} is missing the `{field}` parameter at {location}"))]
    MissingField {
        index: usize,
        field: &'static str,
        location: Location,
    },

    #[snafu(display("JWK #{index} has an invalid base64url `{field}` value at {location}"))]
    InvalidBase64Url {
        index: usize,
        field: &'static str,
        location: Location,
    },

    #[snafu(display("JWK #{index} is not a valid core JWK at {location}"))]
    InvalidJwk {
        index: usize,
        source: serde_json::Error,
        location: Location,
    },
}

/// Converts an internal tenant [`JwkSet`](oceaniam_auth::jwks::JwkSet) into the official
/// [`CoreJsonWebKeySet`], preserving the `kty`/`kid`/`use`/`alg`/`n`/`e` wire values of every
/// public RSA signing key.
pub fn core_jwk_set(jwks: &JwkSet) -> Result<CoreJsonWebKeySet, OidcJwksError> {
    let keys = jwks
        .keys
        .iter()
        .enumerate()
        .map(|(index, jwk)| core_jwk(index, jwk))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CoreJsonWebKeySet::new(keys))
}

/// Converts a single internal JWK into a [`CoreJsonWebKey`].
///
/// `CoreJsonWebKey::new_rsa` cannot express the `alg` parameter (the field is private and the
/// constructor leaves it unset), so the official key is built by deserializing the equivalent
/// JWK JSON. The explicit `kty`/`n`/`e` validation above prevents the crate's lenient option
/// deserialization from silently dropping malformed parameters.
fn core_jwk(index: usize, jwk: &oceaniam_auth::jwks::Jwk) -> Result<CoreJsonWebKey, OidcJwksError> {
    if jwk.kty != "RSA" {
        return Err(OidcJwksError::UnsupportedKeyType {
            index,
            kty: jwk.kty.clone(),
            location: snafu::location!(),
        });
    }

    let n = jwk.n.as_deref().ok_or(OidcJwksError::MissingField {
        index,
        field: "n",
        location: snafu::location!(),
    })?;
    let e = jwk.e.as_deref().ok_or(OidcJwksError::MissingField {
        index,
        field: "e",
        location: snafu::location!(),
    })?;

    URL_SAFE_NO_PAD
        .decode(n)
        .map_err(|_| OidcJwksError::InvalidBase64Url {
            index,
            field: "n",
            location: snafu::location!(),
        })?;
    URL_SAFE_NO_PAD
        .decode(e)
        .map_err(|_| OidcJwksError::InvalidBase64Url {
            index,
            field: "e",
            location: snafu::location!(),
        })?;

    let value = json!({
        "kty": jwk.kty,
        "kid": jwk.kid,
        "use": jwk.use_,
        "alg": jwk.alg,
        "n": n,
        "e": e,
    });

    serde_json::from_value(value).map_err(|source| OidcJwksError::InvalidJwk {
        index,
        source,
        location: snafu::location!(),
    })
}

#[cfg(test)]
mod tests {
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
    use im::vector;

    use super::*;
    use oceaniam_auth::jwks::{Jwk, JwkSet};

    fn sample_modulus() -> String {
        URL_SAFE_NO_PAD.encode([0x01, 0x02, 0x03, 0x04])
    }

    fn ps512_jwk() -> Jwk {
        Jwk {
            kty: "RSA".to_string(),
            kid: Some("key-1".to_string()),
            use_: Some("sig".to_string()),
            alg: Some("PS512".to_string()),
            n: Some(sample_modulus()),
            e: Some("AQAB".to_string()),
        }
    }

    // NOTE: AI-generated test
    #[test]
    fn converts_internal_jwk_set_preserving_all_public_fields() {
        let internal = JwkSet {
            keys: vector![ps512_jwk()],
        };

        let core = core_jwk_set(&internal).expect("PS512 RSA JWK should convert");

        assert_eq!(core.keys().len(), 1);
        let serialized = serde_json::to_value(&core).expect("core set should serialize");
        let key = &serialized["keys"][0];

        assert_eq!(key["kty"], "RSA");
        assert_eq!(key["kid"], "key-1");
        assert_eq!(key["use"], "sig");
        assert_eq!(key["alg"], "PS512");
        assert_eq!(key["n"], sample_modulus());
        assert_eq!(key["e"], "AQAB");
        assert!(
            key.get("d").is_none() && key.get("p").is_none() && key.get("k").is_none(),
            "no private or symmetric key material may be exposed"
        );
    }

    // NOTE: AI-generated test
    #[test]
    fn serializes_as_a_bare_jwk_set_object() {
        let internal = JwkSet {
            keys: vector![ps512_jwk()],
        };

        let core = core_jwk_set(&internal).expect("PS512 RSA JWK should convert");
        let serialized = serde_json::to_value(&core).expect("core set should serialize");

        assert_eq!(serialized.as_object().expect("bare object").len(), 1);
        assert!(serialized.get("keys").is_some());
    }

    // NOTE: AI-generated test
    #[test]
    fn rejects_non_rsa_key_types() {
        let mut jwk = ps512_jwk();
        jwk.kty = "EC".to_string();
        let internal = JwkSet { keys: vector![jwk] };

        let error = core_jwk_set(&internal).expect_err("EC keys must be rejected");

        assert!(
            matches!(error, OidcJwksError::UnsupportedKeyType { index: 0, kty, .. } if kty == "EC")
        );
    }

    // NOTE: AI-generated test
    #[test]
    fn rejects_missing_modulus_and_exponent() {
        let mut without_n = ps512_jwk();
        without_n.n = None;
        let mut without_e = ps512_jwk();
        without_e.e = None;
        let internal = JwkSet {
            keys: vector![without_n, without_e],
        };

        let error = core_jwk_set(&internal).expect_err("missing parameters must be rejected");

        assert!(
            matches!(
                error,
                OidcJwksError::MissingField {
                    index: 0,
                    field: "n",
                    ..
                }
            ),
            "unexpected error: {error}"
        );
    }

    // NOTE: AI-generated test
    #[test]
    fn rejects_invalid_base64url_modulus_instead_of_silently_dropping_it() {
        let mut jwk = ps512_jwk();
        jwk.n = Some("!!!not base64url!!!".to_string());
        let internal = JwkSet { keys: vector![jwk] };

        let error = core_jwk_set(&internal).expect_err("malformed modulus must be rejected");

        assert!(
            matches!(
                error,
                OidcJwksError::InvalidBase64Url {
                    index: 0,
                    field: "n",
                    ..
                }
            ),
            "unexpected error: {error}"
        );
    }

    // NOTE: AI-generated test
    #[test]
    fn rejects_unrecognized_alg_instead_of_silently_dropping_it() {
        let mut jwk = ps512_jwk();
        jwk.alg = Some("NOT-AN-ALG".to_string());
        let internal = JwkSet { keys: vector![jwk] };

        let error = core_jwk_set(&internal).expect_err("unknown alg must be rejected");

        assert!(matches!(error, OidcJwksError::InvalidJwk { index: 0, .. }));
    }

    // NOTE: AI-generated test
    #[test]
    fn accepts_unspecified_alg_and_reports_each_failing_index() {
        let mut without_alg = ps512_jwk();
        without_alg.alg = None;
        let mut ec = ps512_jwk();
        ec.kty = "oct".to_string();
        let keys = vector![without_alg.clone(), ec];
        let internal = JwkSet { keys };

        let core = core_jwk_set(&internal)
            .map(|_| unreachable!("the second key must fail"))
            .expect_err("the second key must fail");

        assert!(
            matches!(core, OidcJwksError::UnsupportedKeyType { index: 1, ref kty, .. } if kty == "oct"),
            "unexpected error: {core}"
        );

        let only_unspecified = JwkSet {
            keys: vector![without_alg],
        };
        let core = core_jwk_set(&only_unspecified).expect("unspecified alg is valid JWK JSON");
        assert_eq!(core.keys().len(), 1);
        let serialized = serde_json::to_value(&core).expect("serialize");
        assert!(
            serialized["keys"][0].get("alg").is_none(),
            "an unspecified algorithm should be omitted from the JWK"
        );
    }
}
