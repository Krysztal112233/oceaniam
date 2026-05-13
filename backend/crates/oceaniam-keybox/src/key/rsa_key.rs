use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use oceaniam_auth::{jwks::Jwk, jwt::JwtCodec};
use oceaniam_database::model::{key_boxes::Model as Key, sea_orm_active_enums::KeyStatus};
use rsa::{
    RsaPrivateKey,
    pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey},
    pkcs8::{DecodePrivateKey, EncodePrivateKey, der::zeroize::Zeroize},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    error::Error,
    key::{AsSecretField, FromSecretField, TryIntoJwk, TryIntoKeyModel},
    key_alg::KeyAlg,
    keybox::{KeyOption, StandaloneKey},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RsaKey {
    key_id: Uuid,
    key_alg: KeyAlg,

    private: RsaPrivateKey,
}

impl RsaKey {
    pub fn new(key_id: Uuid, key_alg: impl Into<KeyAlg>) -> Self {
        Self::with_bit_size(key_id, key_alg, 4096).unwrap()
    }

    pub fn with_bit_size(
        key_id: Uuid,
        key_alg: impl Into<KeyAlg>,
        bit_size: usize,
    ) -> Result<Self, Error> {
        let mut rng = rand::thread_rng();
        let private = RsaPrivateKey::new(&mut rng, bit_size)?;

        Ok(Self {
            private,
            key_alg: key_alg.into(),
            key_id,
        })
    }

    pub fn key_id(&self) -> Uuid {
        self.key_id
    }
}

impl TryIntoJwk for RsaKey {
    fn try_into_jwk(self) -> Result<oceaniam_auth::jwks::Jwk, Error> {
        // NOTE: ONLY SUPPORT PKCS1 DER. WHAT THE FUCK.
        let mut der = self.private.to_pkcs1_der().unwrap().to_bytes();
        let key = EncodingKey::from_rsa_der(&der);
        der.zeroize();

        let mut jwk = jsonwebtoken::jwk::Jwk::from_encoding_key(&key, self.key_alg.into())?;

        jwk.common.key_id = Some(self.key_id.to_string());

        Ok(Jwk::from(jwk))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SecretField {
    /// This field are encoded with [rsa::pkcs8] format.
    pem: String,
}

impl SecretField {
    pub fn from_rsa_private(private: RsaPrivateKey) -> Result<Self, Error> {
        let mut pem = private.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF).unwrap();

        let secret = SecretField {
            pem: pem.to_string(),
        };

        // NOTE: Wipe out from memory
        pem.zeroize();

        Ok(secret)
    }
}

impl AsSecretField for RsaKey {
    fn as_secret_field(&self) -> Result<Value, Error> {
        Ok(serde_json::to_value(SecretField::from_rsa_private(
            self.private.clone(),
        )?)?)
    }
}

impl FromSecretField for RsaKey {
    type Type = RsaPrivateKey;

    fn from_secret_field(value: Value) -> Result<Self::Type, Error> {
        let SecretField { pem } = serde_json::from_value::<SecretField>(value)?;

        Ok(RsaPrivateKey::from_pkcs8_pem(&pem)?)
    }
}

impl TryIntoKeyModel for RsaKey {
    fn try_into_key_model(
        self,
        application_id: Uuid,
        KeyOption {
            created_at,
            activated_at,
            retired_at,
            expires_at,
        }: crate::keybox::KeyOption,
    ) -> Result<oceaniam_database::model::key_boxes::Model, Error> {
        let StandaloneKey {
            key_id: id,
            key_alg,
            secret,
        } = self.try_into()?;

        let status = {
            let now: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();

            if now >= expires_at || now >= retired_at {
                KeyStatus::Retired
            } else if now >= activated_at {
                KeyStatus::Active
            } else {
                KeyStatus::Pending
            }
        };

        Ok(Key {
            id,
            key_alg: key_alg.into(),
            status,
            created_at,
            activated_at,
            retired_at,
            revoked_at: None,
            expires_at,
            secret,
            application_id,
        })
    }
}

impl TryFrom<Key> for RsaKey {
    type Error = Error;

    fn try_from(
        Key {
            id: key_id,
            key_alg,
            secret,
            ..
        }: Key,
    ) -> Result<Self, Self::Error> {
        let key_alg = KeyAlg::from(key_alg);

        Ok(Self {
            key_id,
            key_alg,
            private: Self::from_secret_field(secret)?,
        })
    }
}

impl TryFrom<RsaKey> for StandaloneKey {
    type Error = Error;

    fn try_from(
        RsaKey {
            key_id: id,
            key_alg,
            private: secret,
        }: RsaKey,
    ) -> Result<Self, Self::Error> {
        Ok(StandaloneKey {
            key_id: id,
            key_alg,
            secret: serde_json::to_value(SecretField::from_rsa_private(secret)?)?,
        })
    }
}

impl TryFrom<StandaloneKey> for RsaKey {
    type Error = Error;

    fn try_from(
        StandaloneKey {
            key_id: id,
            key_alg,
            secret,
        }: StandaloneKey,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            key_id: id,
            key_alg,
            private: Self::from_secret_field(secret)?,
        })
    }
}

impl<T> JwtCodec<T> for RsaKey
where
    T: DeserializeOwned + Serialize,
{
    fn encode(&self, header: Header, claim: T) -> Result<String, oceaniam_auth::error::Error> {
        let der = self
            .private
            .to_pkcs1_der()
            .map_err(|_| oceaniam_auth::error::Error::Jwt {
                source: jsonwebtoken::errors::new_error(
                    jsonwebtoken::errors::ErrorKind::InvalidKeyFormat,
                ),
            })?;
        let key = EncodingKey::from_rsa_der(der.as_bytes());

        Ok(encode(&header, &claim, &key)?)
    }

    fn decode(
        &self,
        jwt: &[u8],
        validation: &Validation,
    ) -> Result<jsonwebtoken::TokenData<T>, oceaniam_auth::error::Error> {
        let der = self.private.to_public_key().to_pkcs1_der().map_err(|_| {
            oceaniam_auth::error::Error::Jwt {
                source: jsonwebtoken::errors::new_error(
                    jsonwebtoken::errors::ErrorKind::InvalidKeyFormat,
                ),
            }
        })?;
        let key = DecodingKey::from_rsa_der(der.as_bytes());

        Ok(decode(jwt, &key, validation)?)
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use jsonwebtoken::{Algorithm, TokenData};
    use oceaniam_auth::jwt::{ClaimHelper, SystemClaim};
    use tap::Tap;
    use uuid::Uuid;

    use super::*;

    const SUPPORTED_ALGORITHM: &[Algorithm] = &[
        Algorithm::PS256,
        Algorithm::PS384,
        Algorithm::PS512,
        Algorithm::RS256,
        Algorithm::RS384,
        Algorithm::RS512,
    ];

    #[test]
    fn test_rsa_key_into_jwks() {
        assert!(
            SUPPORTED_ALGORITHM
                .iter()
                .map(
                    |alg| RsaKey::new(Uuid::now_v7(), KeyAlg::try_from(*alg).unwrap())
                        .try_into_jwk()
                )
                .all(|it| it.is_ok())
        )
    }

    #[test]
    fn test_rsa_as_secret_field() {
        for alg in SUPPORTED_ALGORITHM.iter() {
            let key = RsaKey::new(Uuid::now_v7(), KeyAlg::try_from(*alg).unwrap());
            let secret = key.as_secret_field();
            assert!(secret.is_ok());
            assert!(RsaKey::from_secret_field(secret.unwrap()).is_ok());
        }
    }

    #[test]
    fn test_rsa_as_standalone_key() {
        for alg in SUPPORTED_ALGORITHM.iter() {
            let key = RsaKey::new(Uuid::now_v7(), KeyAlg::try_from(*alg).unwrap());
            assert!(StandaloneKey::try_from(key).is_ok())
        }
    }

    #[test]
    fn test_jwt_codec() -> Result<(), Error> {
        let key = RsaKey::new(Uuid::now_v7(), KeyAlg::try_from(Algorithm::RS512).unwrap());

        let jwt = key
            .encode(
                Header::new(Algorithm::RS512),
                SystemClaim::new(Uuid::now_v7(), 60, None, None),
            )
            .unwrap();

        let _: TokenData<SystemClaim> = key
            .decode(
                jwt.as_bytes(),
                &Validation::default().tap_mut(|it| {
                    it.algorithms = SUPPORTED_ALGORITHM.iter().cloned().collect_vec()
                }),
            )
            .unwrap();

        Ok(())
    }

    #[test]
    fn test_rsa_as_jwk() {
        let key = RsaKey::new(Uuid::now_v7(), KeyAlg::try_from(Algorithm::RS512).unwrap());

        assert!(key.try_into_jwk().is_ok())
    }
}
