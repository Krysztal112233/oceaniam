use oceaniam_crypto::Algorithm;
use oceaniam_database::model::sea_orm_active_enums::KeyAlg as DatabaseKeyAlgorithm;
use serde::{Deserialize, Serialize};

use crate::error::Error;

/// The single source of truth for the tenant signing key algorithm.
///
/// Every newly generated tenant key uses this algorithm, so extant fleets converge to it
/// through natural rotation and a future algorithm migration is a single-constant bump.
pub const TENANT_KEY_ALG: KeyAlg = KeyAlg(DatabaseKeyAlgorithm::Rs256);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(transparent)]
pub struct KeyAlg(DatabaseKeyAlgorithm);

macro_rules! alg_conversions {
    ($(($jwt:ident, $db:ident)),+ $(,)?) => {
        impl TryFrom<Algorithm> for KeyAlg {
            type Error = Error;

            fn try_from(value: Algorithm) -> Result<Self, Self::Error> {
                match value {
                    $(Algorithm::$jwt => Ok(Self(DatabaseKeyAlgorithm::$db)),)+
                    _ => Err(Error::unimplemented_jwt_alogrithm(value)),
                }
            }
        }

        impl From<KeyAlg> for Algorithm {
            fn from(value: KeyAlg) -> Self {
                match value.0 {
                    $(DatabaseKeyAlgorithm::$db => Algorithm::$jwt),+
                }
            }
        }

        impl KeyAlg {
            /// The JWA algorithm name (RFC 7518 §3.1), as carried in JWT headers and
            /// advertised in discovery metadata.
            pub fn as_str(&self) -> &'static str {
                match self.0 {
                    $(DatabaseKeyAlgorithm::$db => stringify!($jwt),)+
                }
            }
        }
    };
}

alg_conversions!(
    (RS256, Rs256),
    (RS384, Rs384),
    (RS512, Rs512),
    (PS256, Ps256),
    (PS384, Ps384),
    (PS512, Ps512),
);

impl From<DatabaseKeyAlgorithm> for KeyAlg {
    fn from(value: DatabaseKeyAlgorithm) -> Self {
        Self(value)
    }
}

impl From<KeyAlg> for DatabaseKeyAlgorithm {
    fn from(value: KeyAlg) -> Self {
        value.0
    }
}

impl std::ops::Deref for KeyAlg {
    type Target = DatabaseKeyAlgorithm;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: AI-generated test
    #[test]
    fn as_str_matches_the_jwa_name_for_every_supported_variant() {
        // The macro pairs are hand-maintained; pin all six against the JWA name and
        // cross-check against the serde serialization of the jsonwebtoken conversion
        // (precedent: oceaniam-crypto/tests/aws_lc_compatibility.rs).
        let matrix: [(DatabaseKeyAlgorithm, &str); 6] = [
            (DatabaseKeyAlgorithm::Rs256, "RS256"),
            (DatabaseKeyAlgorithm::Rs384, "RS384"),
            (DatabaseKeyAlgorithm::Rs512, "RS512"),
            (DatabaseKeyAlgorithm::Ps256, "PS256"),
            (DatabaseKeyAlgorithm::Ps384, "PS384"),
            (DatabaseKeyAlgorithm::Ps512, "PS512"),
        ];
        for (variant, jwa_name) in matrix {
            let alg = KeyAlg::from(variant.clone());
            assert_eq!(alg.as_str(), jwa_name, "as_str for {variant:?}");
            assert_eq!(
                serde_json::to_value(Algorithm::from(alg)).expect("algorithm serializes"),
                serde_json::Value::String(jwa_name.to_owned()),
                "serde serialization of the jsonwebtoken conversion for {variant:?}"
            );
        }
    }
}
