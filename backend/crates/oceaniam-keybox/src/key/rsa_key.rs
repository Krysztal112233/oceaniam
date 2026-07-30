use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use oceaniam_auth::{jwks::Jwk, jwt::JwtCodec};
use oceaniam_common::{
    consts,
    crypto::{EncryptedBlob, MasterKey},
    run_cpu_bound,
};
use oceaniam_database::model::key_boxes::Model as Key;
use rsa::{
    RsaPrivateKey,
    pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey},
    pkcs8::{DecodePrivateKey, EncodePrivateKey, der::zeroize::Zeroize},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use tracing::field::Empty;
use uuid::Uuid;

use crate::{
    error::Error,
    key::{FromSecretField, TryIntoJwk, TryIntoKeyModel},
    key_alg::KeyAlg,
    keybox::{KeyOption, RawKey, compute_key_status},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RsaKey {
    key_id: Uuid,
    key_alg: KeyAlg,

    pub(crate) private: RsaPrivateKey,
}

impl RsaKey {
    pub fn new(key_id: Uuid, key_alg: impl Into<KeyAlg>) -> Self {
        Self::with_bit_size(key_id, key_alg, 4096).unwrap()
    }

    pub async fn generate(key_id: Uuid, key_alg: impl Into<KeyAlg>) -> Result<Self, Error> {
        Self::generate_with_bit_size(key_id, key_alg.into(), 4096).await
    }

    async fn generate_with_bit_size(
        key_id: Uuid,
        key_alg: KeyAlg,
        bit_size: usize,
    ) -> Result<Self, Error> {
        let queue_span = tracing::info_span!(
            "keybox.rsa.queue",
            otel.kind = "internal",
            cpu.operation = "rsa.generate",
            rsa.bits = bit_size,
            rsa.algorithm = ?key_alg,
        );

        run_cpu_bound(queue_span, move |parent| {
            let span = tracing::info_span!(
                parent: &parent,
                "keybox.rsa.generate",
                otel.kind = "internal",
                otel.status_code = Empty,
                otel.status_description = Empty,
                rsa.bits = bit_size,
                rsa.algorithm = ?key_alg,
            );
            let result = span.in_scope(|| Self::with_bit_size(key_id, key_alg, bit_size));
            if result.is_err() {
                span.record("otel.status_code", "ERROR");
                span.record("otel.status_description", "RSA key generation failed");
            }
            result
        })
        .await?
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

    pub fn key_alg(&self) -> KeyAlg {
        self.key_alg.clone()
    }
}

impl TryIntoJwk for RsaKey {
    #[tracing::instrument(
        level = "info",
        name = "keybox.rsa.to_jwk",
        skip_all,
        fields(otel.kind = "internal")
    )]
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
pub struct SecretField {
    /// base64-encoded 24-byte XChaCha20 nonce.
    pub nonce: String,
    /// base64-encoded ciphertext (encrypted PKCS#8 PEM + Poly1305 tag).
    pub ciphertext: String,
    /// KEK version that produced this ciphertext.
    pub key_version: u32,
}

impl SecretField {
    #[tracing::instrument(
        level = "info",
        name = "keybox.private_key.seal",
        skip_all,
        fields(otel.kind = "internal")
    )]
    pub fn from_rsa_private(private: RsaPrivateKey, master_key: &MasterKey) -> Result<Self, Error> {
        use base64::{Engine as _, engine::general_purpose::STANDARD as B64};

        let mut pem = private.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)?;

        let blob = master_key.encrypt(pem.as_bytes(), consts::KEK_VERSION_CURRENT)?;

        // zeroize the PEM buffer
        pem.zeroize();

        Ok(Self {
            nonce: B64.encode(blob.nonce),
            ciphertext: B64.encode(&blob.ciphertext),
            key_version: blob.key_version,
        })
    }
}

impl FromSecretField for RsaKey {
    type Type = RsaPrivateKey;

    #[tracing::instrument(
        level = "info",
        name = "keybox.private_key.unseal",
        skip_all,
        fields(otel.kind = "internal")
    )]
    fn from_secret_field(value: Value, master_key: &MasterKey) -> Result<Self::Type, Error> {
        use base64::{Engine as _, engine::general_purpose::STANDARD as B64};

        let field: SecretField = serde_json::from_value(value)?;

        let nonce: [u8; 24] = B64
            .decode(&field.nonce)
            .map_err(|e| Error::Internal {
                msg: format!("nonce base64 decode: {e}"),
                location: snafu::location!(),
            })?
            .try_into()
            .map_err(|_| Error::Internal {
                msg: "nonce must be exactly 24 bytes".to_string(),
                location: snafu::location!(),
            })?;

        let blob = EncryptedBlob {
            nonce,
            ciphertext: B64.decode(&field.ciphertext).map_err(|e| Error::Internal {
                msg: format!("ciphertext base64 decode: {e}"),
                location: snafu::location!(),
            })?,
            key_version: field.key_version,
        };

        let pem_bytes = master_key.decrypt(&blob)?;
        let pem_str = String::from_utf8(pem_bytes).map_err(|e| Error::Internal {
            msg: format!("decrypted PEM is not valid UTF-8: {e}"),
            location: snafu::location!(),
        })?;

        Ok(RsaPrivateKey::from_pkcs8_pem(&pem_str)?)
    }
}

impl TryIntoKeyModel for RsaKey {
    fn try_into_key_model(
        self,
        tenant_id: Uuid,
        master_key: &MasterKey,
        KeyOption {
            created_at,
            activated_at,
            retired_at,
            expires_at,
        }: crate::keybox::KeyOption,
    ) -> Result<oceaniam_database::model::key_boxes::Model, Error> {
        let secret =
            serde_json::to_value(SecretField::from_rsa_private(self.private, master_key)?)?;

        let status = {
            let now: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
            compute_key_status(&now, &activated_at, &retired_at, &expires_at)
        };

        Ok(Key {
            id: self.key_id,
            key_alg: self.key_alg.into(),
            status,
            created_at,
            activated_at,
            retired_at,
            revoked_at: None,
            expires_at,
            secret,
            tenant_id,
        })
    }
}

impl RsaKey {
    /// Decrypt the private key from a DB `Key` model.
    #[tracing::instrument(
        level = "info",
        name = "keybox.rsa.from_model",
        skip_all,
        fields(otel.kind = "internal")
    )]
    pub fn from_key(key: Key, master_key: &MasterKey) -> Result<Self, Error> {
        let Key {
            id: key_id,
            key_alg,
            secret,
            ..
        } = key;
        let key_alg = KeyAlg::from(key_alg);

        Ok(Self {
            key_id,
            key_alg,
            private: Self::from_secret_field(secret, master_key)?,
        })
    }

    /// Encrypt this key into a `RawKey` for storage.
    #[tracing::instrument(
        level = "info",
        name = "keybox.rsa.into_raw",
        skip_all,
        fields(otel.kind = "internal")
    )]
    pub fn into_raw_key(self, master_key: &MasterKey) -> Result<RawKey, Error> {
        let RsaKey {
            key_id: id,
            key_alg,
            private: secret,
        } = self;

        Ok(RawKey {
            key_id: id,
            key_alg,
            secret: serde_json::to_value(SecretField::from_rsa_private(secret, master_key)?)?,
        })
    }

    /// Decrypt the private key from a `RawKey`.
    #[tracing::instrument(
        level = "info",
        name = "keybox.rsa.from_raw",
        skip_all,
        fields(otel.kind = "internal")
    )]
    pub fn from_raw_key(raw: RawKey, master_key: &MasterKey) -> Result<Self, Error> {
        let RawKey {
            key_id: id,
            key_alg,
            secret,
        } = raw;

        Ok(Self {
            key_id: id,
            key_alg,
            private: Self::from_secret_field(secret, master_key)?,
        })
    }
}

impl<T> JwtCodec<T> for RsaKey
where
    T: DeserializeOwned + Serialize,
{
    #[tracing::instrument(
        level = "info",
        name = "auth.jwt.encode",
        skip_all,
        fields(otel.kind = "internal", key.id = %self.key_id, jwt.algorithm = ?self.key_alg)
    )]
    fn encode(&self, header: Header, claim: T) -> Result<String, oceaniam_auth::error::Error> {
        let der = self
            .private
            .to_pkcs1_der()
            .map_err(|_| oceaniam_auth::error::Error::Jwt {
                source: jsonwebtoken::errors::new_error(
                    jsonwebtoken::errors::ErrorKind::InvalidKeyFormat,
                ),
                location: snafu::location!(),
            })?;
        let key = EncodingKey::from_rsa_der(der.as_bytes());

        Ok(encode(&header, &claim, &key)?)
    }

    #[tracing::instrument(
        level = "info",
        name = "auth.jwt.decode",
        skip_all,
        fields(otel.kind = "internal", key.id = %self.key_id, jwt.algorithm = ?self.key_alg)
    )]
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
                location: snafu::location!(),
            }
        })?;
        let key = DecodingKey::from_rsa_der(der.as_bytes());

        Ok(decode(jwt, &key, validation)?)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};
    use std::sync::{Arc, Mutex};

    use itertools::Itertools;
    use jsonwebtoken::{Algorithm, TokenData};
    use oceaniam_auth::jwt::{ClaimHelper, SystemClaim};
    use tap::Tap;
    use tracing::Instrument as _;
    use tracing_subscriber::fmt::format::FmtSpan;
    use uuid::Uuid;

    use super::*;

    #[derive(Clone)]
    struct Buffer(Arc<Mutex<Vec<u8>>>);

    impl Write for Buffer {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0
                .lock()
                .expect("trace buffer lock")
                .extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

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

    // NOTE: AI-generated test
    #[test]
    fn test_rsa_as_standalone_key() {
        let mk =
            MasterKey::from_hex("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .unwrap();
        for alg in SUPPORTED_ALGORITHM.iter() {
            let key = RsaKey::new(Uuid::now_v7(), KeyAlg::try_from(*alg).unwrap());
            assert!(key.into_raw_key(&mk).is_ok())
        }
    }

    // NOTE: AI-generated test
    #[test]
    fn secret_field_round_trip_preserves_key() {
        use rsa::traits::PublicKeyParts;

        let mk =
            MasterKey::from_hex("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .unwrap();
        let original = RsaKey::new(Uuid::now_v7(), KeyAlg::try_from(Algorithm::RS512).unwrap());

        let raw = original.clone().into_raw_key(&mk).unwrap();
        let recovered = RsaKey::from_raw_key(raw, &mk).unwrap();

        // Compare by modulus (RsaPrivateKey doesn't impl Eq)
        let orig_n = original.private.n();
        let recv_n = recovered.private.n();
        assert_eq!(orig_n, recv_n);
    }

    // NOTE: AI-generated test
    #[test]
    fn secret_field_output_is_valid_base64() {
        let mk =
            MasterKey::from_hex("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .unwrap();
        use base64::{Engine as _, engine::general_purpose::STANDARD as B64};

        let key = RsaKey::new(Uuid::now_v7(), KeyAlg::try_from(Algorithm::RS512).unwrap());
        let raw = key.into_raw_key(&mk).unwrap();

        let secret: SecretField = serde_json::from_value(raw.secret).unwrap();
        assert!(B64.decode(&secret.nonce).is_ok());
        assert!(B64.decode(&secret.ciphertext).is_ok());
    }

    // NOTE: AI-generated test
    #[test]
    fn secret_field_key_version_is_one() {
        let mk =
            MasterKey::from_hex("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .unwrap();
        let key = RsaKey::new(Uuid::now_v7(), KeyAlg::try_from(Algorithm::RS512).unwrap());
        let raw = key.into_raw_key(&mk).unwrap();

        let secret: SecretField = serde_json::from_value(raw.secret).unwrap();
        assert_eq!(secret.key_version, 1);
    }

    // NOTE: AI-generated test
    #[test]
    fn two_encryptions_produce_different_nonces() {
        let mk =
            MasterKey::from_hex("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .unwrap();
        let key = RsaKey::new(Uuid::now_v7(), KeyAlg::try_from(Algorithm::RS512).unwrap());

        let raw1 = key.clone().into_raw_key(&mk).unwrap();
        let raw2 = key.into_raw_key(&mk).unwrap();

        let s1: SecretField = serde_json::from_value(raw1.secret).unwrap();
        let s2: SecretField = serde_json::from_value(raw2.secret).unwrap();
        assert_ne!(s1.nonce, s2.nonce);
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

    // NOTE: AI-generated test
    #[tokio::test]
    async fn rsa_generation_span_crosses_blocking_dispatch() {
        let bytes = Arc::new(Mutex::new(Vec::new()));
        let writer = bytes.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(move || Buffer(writer.clone()))
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .finish();
        let dispatch = tracing::Dispatch::new(subscriber);
        let _default = tracing::dispatcher::set_default(&dispatch);

        async {
            RsaKey::generate_with_bit_size(
                Uuid::now_v7(),
                KeyAlg::try_from(Algorithm::RS512).expect("supported algorithm"),
                1024,
            )
            .await
            .expect("generate test key");
        }
        .instrument(tracing::info_span!("test.request"))
        .await;

        let output = String::from_utf8(bytes.lock().expect("trace buffer lock").clone())
            .expect("utf8 trace output");
        assert!(output.contains("keybox.rsa.queue"));
        assert!(output.contains("keybox.rsa.generate"));
        assert!(output.contains("test.request"));
    }
}
