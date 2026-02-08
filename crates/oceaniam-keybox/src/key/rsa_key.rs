use jsonwebtoken::{Algorithm, EncodingKey, jwk::Jwk};
use oceaniam_database::model::key_boxes::Model as Key;
use rsa::{
    RsaPrivateKey,
    pkcs1::EncodeRsaPrivateKey,
    pkcs8::{DecodePrivateKey, EncodePrivateKey, der::zeroize::Zeroize},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    error::Error,
    key::{AsSecretField, FromSecretField},
    key_alg::KeyAlg,
    key_box::StandloneKey,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RsaKey {
    key_id: Uuid,
    key_alg: KeyAlg,

    private: RsaPrivateKey,
}

impl RsaKey {
    pub fn new(key_id: Uuid, key_alg: KeyAlg) -> Self {
        Self::with_bit_size(key_id, key_alg, 4096).unwrap()
    }

    pub fn with_bit_size(key_id: Uuid, key_alg: KeyAlg, bit_size: usize) -> Result<Self, Error> {
        let mut rng = rand::thread_rng();
        let private = RsaPrivateKey::new(&mut rng, bit_size)?;

        Ok(Self {
            private,
            key_alg,
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

        // Wipe out from memory
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

impl TryFrom<RsaKey> for StandloneKey {
    type Error = Error;

    fn try_from(
        RsaKey {
            key_id: id,
            key_alg,
            private: secret,
        }: RsaKey,
    ) -> Result<Self, Self::Error> {
        Ok(StandloneKey {
            id,
            key_alg,
            secret: serde_json::to_value(SecretField::from_rsa_private(secret)?)?,
        })
    }
}

#[cfg(test)]
mod tests {
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
            assert!(StandloneKey::try_from(key).is_ok())
        }
    }
}
