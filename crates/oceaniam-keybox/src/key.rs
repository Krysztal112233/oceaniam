use jsonwebtoken::{Algorithm, EncodingKey, jwk::Jwk};
use oceaniam_database::model::sea_orm_active_enums::KeyAlg;
use rsa::{RsaPrivateKey, pkcs1::EncodeRsaPrivateKey, pkcs8::der::zeroize::Zeroize};
use serde::{Deserialize, Serialize};

use crate::error::Error;

type Key = oceaniam_database::model::key_boxes::Model;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RSAKey {
    kid: String,
    private: RsaPrivateKey,
}

impl RSAKey {
    pub fn new(key_id: impl Into<String>) -> Self {
        Self::with_bit_size(key_id, 4096).unwrap()
    }

    pub fn with_bit_size(key_id: impl Into<String>, bit_size: usize) -> Result<Self, Error> {
        let mut rng = rand::thread_rng();
        let private = RsaPrivateKey::new(&mut rng, bit_size)?;

        Ok(Self {
            private,
            kid: key_id.into(),
        })
    }

    pub fn into_jwk(self, alg: Algorithm) -> Result<Jwk, Error> {
        let mut der = self.private.to_pkcs1_der().unwrap().to_bytes();
        let key = EncodingKey::from_rsa_der(&der);
        der.zeroize();
        Ok(Jwk::from_encoding_key(&key, alg)?)
    }
}

impl TryFrom<Key> for RSAKey {
    type Error = Error;

    fn try_from(value: Key) -> Result<Self, Self::Error> {
        #[allow(unreachable_patterns)]
        match value.key_alg {
            KeyAlg::Rs256
            | KeyAlg::Rs384
            | KeyAlg::Rs512
            | KeyAlg::Ps256
            | KeyAlg::Ps384
            | KeyAlg::Ps512 => Ok(Self {
                kid: value.key_id.to_string(),
                private: serde_json::from_value(value.secret)?,
            }),
            _ => Err(Error::MismatchedKeyAlg(value.key_alg)),
        }
    }
}

#[cfg(test)]
mod tests {

    use uuid::Uuid;

    use super::*;

    #[test]
    fn test_rsa_key_pair_into_jwk() {
        assert!(
            [
                Algorithm::PS256,
                Algorithm::PS384,
                Algorithm::PS512,
                Algorithm::RS256,
                Algorithm::RS384,
                Algorithm::RS512,
            ]
            .into_iter()
            .map(|alg| RSAKey::new(Uuid::now_v7()).into_jwk(alg))
            .all(|it| it.is_ok())
        )
    }
}
