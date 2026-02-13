use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode, jwk::Jwk,
};
use oceaniam_common::jwt::JwtCodec;
use oceaniam_database::model::key_boxes::Model as Key;
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
    key::{AsSecretField, FromSecretField},
    key_alg::KeyAlg,
    keybox::StandaloneKey,
};

#[derive(Debug, Clone)]
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

    pub fn into_jwk(self, alg: Algorithm) -> Result<Jwk, Error> {
        // NOTE: ONLY SUPPORT PKCS1 DER. WHAT THE FUCK.
        let mut der = self.private.to_pkcs1_der().unwrap().to_bytes();
        let key = EncodingKey::from_rsa_der(&der);
        der.zeroize();

        Ok(Jwk::from_encoding_key(&key, alg)?)
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
    fn encode(&self, header: Header, claim: T) -> Result<String, oceaniam_common::error::Error> {
        let key = EncodingKey::from_rsa_der(self.private.to_pkcs1_der()?.as_bytes());

        Ok(encode(&header, &claim, &key)?)
    }

    fn decode(
        &self,
        jwt: &[u8],
        validation: &Validation,
    ) -> Result<jsonwebtoken::TokenData<T>, oceaniam_common::error::Error> {
        let key =
            DecodingKey::from_rsa_der(self.private.to_public_key().to_pkcs1_der()?.as_bytes());

        Ok(decode(jwt, &key, validation)?)
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use jsonwebtoken::TokenData;
    use oceaniam_common::jwt::{ClaimHelper, SystemClaim};
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
                        .into_jwk(*alg)
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
    fn test_rsa_as_standlone_key() {
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
}
