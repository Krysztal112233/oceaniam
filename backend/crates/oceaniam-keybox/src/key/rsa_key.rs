use chrono::Utc;
use oceaniam_auth::jwt::JwtCodec;
use oceaniam_common::{
    consts,
    crypto::{EncryptedBlob, MasterKey},
    run_cpu_bound,
};
use oceaniam_crypto::{
    Header, RsaPrivateKey, TokenData, Validation, decode_rsa_der, encode_rsa, rsa_public_jwk,
};
use oceaniam_database::model::key_boxes::Model as Key;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use tracing::field::Empty;
use uuid::Uuid;
use zeroize::Zeroizing;

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
        let private = RsaPrivateKey::generate(bit_size)?;

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
        let mut jwk = rsa_public_jwk(&self.private, self.key_alg.into())?;
        jwk.common.key_id = Some(self.key_id.to_string());
        let jwk = serde_json::to_value(jwk)?;

        Ok(serde_json::from_value(jwk)?)
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

        let pem = private.to_pkcs8_pem()?;
        let blob = master_key.encrypt(pem.as_bytes(), consts::KEK_VERSION_CURRENT)?;

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

        let pem_bytes = Zeroizing::new(master_key.decrypt(&blob)?);
        Ok(RsaPrivateKey::from_pkcs8_pem(&pem_bytes)?)
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
        Ok(encode_rsa(&header, &claim, &self.private)?)
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
    ) -> Result<TokenData<T>, oceaniam_auth::error::Error> {
        let der = self.private.public_key_der();
        Ok(decode_rsa_der(jwt, &der, validation)?)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use itertools::Itertools;
    use oceaniam_auth::{
        Algorithm, TokenData, Validation, decode, decode_header,
        jwks::JwkSet,
        jwt::{ClaimHelper, SystemClaim},
    };
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

    fn fixture_validation(algorithm: Algorithm) -> Validation {
        let mut validation = Validation::new(algorithm);
        validation.set_issuer(&["https://legacy.oceaniam.test"]);
        validation.set_audience(&["legacy-client"]);
        validation
    }

    fn altered_segment(token: &str, index: usize) -> String {
        let mut segments = token.split('.').map(str::to_owned).collect::<Vec<_>>();
        let mut bytes = segments[index].clone().into_bytes();
        bytes[0] = if bytes[0] == b'A' { b'B' } else { b'A' };
        segments[index] = String::from_utf8(bytes).expect("base64url remains UTF-8");
        segments.join(".")
    }

    fn assert_fixture_claims(actual: &SystemClaim) {
        assert_eq!(
            actual.sub,
            Uuid::parse_str("018f47a6-7b53-7cc0-8f5c-5c8bf2d6b8cf").expect("fixture subject UUID")
        );
        assert_eq!(actual.exp, 4_102_444_800);
        assert_eq!(actual.iat, 1_700_000_000);
        assert_eq!(actual.iss.as_deref(), Some("https://legacy.oceaniam.test"));
        assert_eq!(actual.aud.as_ref().map(Vec::len), Some(1));
        assert_eq!(
            actual
                .aud
                .as_ref()
                .and_then(|audience| audience.first())
                .map(String::as_str),
            Some("legacy-client")
        );
        assert_eq!(
            actual.jti,
            Uuid::parse_str("018f47a6-a340-75f0-94b9-03562a28a721").expect("fixture JTI UUID")
        );
    }

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
        let mk =
            MasterKey::from_hex("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .unwrap();
        let original = RsaKey::new(Uuid::now_v7(), KeyAlg::try_from(Algorithm::RS512).unwrap());

        let raw = original.clone().into_raw_key(&mk).unwrap();
        let recovered = RsaKey::from_raw_key(raw, &mk).unwrap();

        assert_eq!(original.private, recovered.private);
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
    #[test]
    fn legacy_encrypted_pkcs8_fixture_remains_usable_and_tamper_evident() {
        use base64::{Engine as _, engine::general_purpose::STANDARD as B64};

        let master_key =
            MasterKey::from_hex("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .expect("test master key");
        let key_id =
            Uuid::parse_str("018f47a5-f53c-7bd0-bad8-764ae9bac80b").expect("fixture key UUID");
        let secret: Value =
            serde_json::from_str(include_str!("../../tests/fixtures/legacy-rsa-secret.json"))
                .expect("parse legacy encrypted secret");
        let raw = RawKey {
            key_id,
            key_alg: KeyAlg::try_from(Algorithm::PS512).expect("supported algorithm"),
            secret: secret.clone(),
        };
        let now: chrono::DateTime<chrono::FixedOffset> = Utc::now().into();
        let recovered_from_model = RsaKey::from_key(
            Key {
                id: key_id,
                key_alg: raw.key_alg.clone().into(),
                status: oceaniam_database::model::sea_orm_active_enums::KeyStatus::Active,
                created_at: now,
                activated_at: now,
                retired_at: now + chrono::Duration::days(30),
                revoked_at: None,
                expires_at: now + chrono::Duration::days(60),
                secret: secret.clone(),
                tenant_id: Uuid::now_v7(),
            },
            &master_key,
        )
        .expect("unseal legacy PKCS#8 fixture from database model");
        let recovered =
            RsaKey::from_raw_key(raw.clone(), &master_key).expect("unseal legacy PKCS#8 fixture");

        assert_eq!(
            recovered.private, recovered_from_model.private,
            "database model and raw-key paths must recover the same legacy key"
        );
        assert_eq!(
            recovered.private.public_key_der(),
            include_bytes!(
                "../../../oceaniam-crypto/tests/fixtures/legacy-rust-crypto/public-key.pkcs1.der"
            )
        );
        assert_eq!(
            recovered
                .private
                .to_pkcs8_pem()
                .expect("export restored fixture PKCS#8 PEM")
                .as_bytes(),
            include_bytes!(
                "../../../oceaniam-crypto/tests/fixtures/legacy-rust-crypto/private-key.pkcs8.pem"
            )
        );
        assert_eq!(recovered.key_id(), key_id);
        assert_eq!(recovered.key_alg(), raw.key_alg);
        let expected_jwk: Value = serde_json::from_str(include_str!(
            "../../../oceaniam-crypto/tests/fixtures/legacy-rust-crypto/public-jwk.json"
        ))
        .expect("parse expected public JWK");
        let legacy_tokens: BTreeMap<String, String> = serde_json::from_str(include_str!(
            "../../../oceaniam-crypto/tests/fixtures/legacy-rust-crypto/tokens.json"
        ))
        .expect("parse legacy token fixtures");
        let unrelated = RsaKey::with_bit_size(
            key_id,
            KeyAlg::try_from(Algorithm::RS256).expect("supported fixture algorithm"),
            2048,
        )
        .expect("generate unrelated verification key");

        for algorithm in SUPPORTED_ALGORITHM {
            let algorithm_name = format!("{algorithm:?}");
            let mut key = recovered.clone();
            key.key_alg = KeyAlg::try_from(*algorithm).expect("supported fixture algorithm");
            let actual_jwk = key
                .clone()
                .try_into_jwk()
                .expect("extract fixture public JWK");
            assert_eq!(actual_jwk.kty, expected_jwk["kty"]);
            assert_eq!(
                actual_jwk.kid.as_deref(),
                Some(key_id.to_string()).as_deref()
            );
            assert_eq!(actual_jwk.alg.as_deref(), Some(algorithm_name.as_str()));
            assert_eq!(actual_jwk.n.as_deref(), expected_jwk["n"].as_str());
            assert_eq!(actual_jwk.e.as_deref(), expected_jwk["e"].as_str());
            assert!(actual_jwk.use_.is_none());

            let jwks = JwkSet {
                keys: std::iter::once(actual_jwk).collect(),
            };
            let decoding_key = jwks
                .decoding_key_for_kid(&key_id.to_string())
                .expect("construct verification key from published JWK");
            assert!(jwks.decoding_key_for_kid("unknown-key").is_err());
            let validation = fixture_validation(*algorithm);
            let legacy_token = legacy_tokens
                .get(&algorithm_name)
                .expect("legacy token for every supported algorithm");

            let legacy_local: TokenData<SystemClaim> = key
                .decode(legacy_token.as_bytes(), &validation)
                .expect("verify legacy token through RsaKey");
            assert_fixture_claims(&legacy_local.claims);
            let legacy_published: TokenData<SystemClaim> =
                decode(legacy_token, &decoding_key, &validation)
                    .expect("verify legacy token through published JWK");
            assert_fixture_claims(&legacy_published.claims);

            let mut header = Header::new(*algorithm);
            header.kid = Some(key_id.to_string());
            let token = key
                .encode(header, legacy_local.claims.clone())
                .expect("sign through restored RsaKey");
            if matches!(
                algorithm,
                Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512
            ) {
                assert_eq!(&token, legacy_token);
            }
            let decoded_local: TokenData<SystemClaim> = key
                .decode(token.as_bytes(), &validation)
                .expect("verify new token through RsaKey");
            assert_fixture_claims(&decoded_local.claims);
            let decoded_published: TokenData<SystemClaim> =
                decode(&token, &decoding_key, &validation)
                    .expect("verify new token through published JWK");
            assert_fixture_claims(&decoded_published.claims);
            assert_eq!(decoded_published.header.alg, *algorithm);
            assert_eq!(
                decoded_published.header.kid.as_deref(),
                Some(key_id.to_string()).as_deref()
            );

            assert!(
                decode::<SystemClaim>(&altered_segment(&token, 1), &decoding_key, &validation)
                    .is_err()
            );
            assert!(
                decode::<SystemClaim>(&altered_segment(&token, 2), &decoding_key, &validation)
                    .is_err()
            );

            let mut wrong_key = unrelated.clone();
            wrong_key.key_alg = KeyAlg::try_from(*algorithm).expect("supported fixture algorithm");
            let wrong_local: Result<TokenData<SystemClaim>, _> =
                wrong_key.decode(token.as_bytes(), &validation);
            assert!(wrong_local.is_err());
            let wrong_jwk = wrong_key
                .try_into_jwk()
                .expect("extract unrelated public JWK");
            let wrong_jwks = JwkSet {
                keys: std::iter::once(wrong_jwk).collect(),
            };
            let wrong_decoding_key = wrong_jwks
                .decoding_key_for_kid(&key_id.to_string())
                .expect("construct unrelated JWK verification key");
            assert!(decode::<SystemClaim>(&token, &wrong_decoding_key, &validation).is_err());

            let excluded_algorithm = if *algorithm == Algorithm::RS256 {
                Algorithm::RS384
            } else {
                Algorithm::RS256
            };
            assert!(
                decode::<SystemClaim>(
                    &token,
                    &decoding_key,
                    &fixture_validation(excluded_algorithm),
                )
                .is_err()
            );

            let without_kid = key
                .encode(Header::new(*algorithm), legacy_local.claims)
                .expect("sign header without key ID");
            assert!(
                decode_header(&without_kid)
                    .expect("decode header without key ID")
                    .kid
                    .is_none()
            );
        }

        let resealed = recovered
            .clone()
            .into_raw_key(&master_key)
            .expect("reseal restored key");
        let recovered_again =
            RsaKey::from_raw_key(resealed, &master_key).expect("unseal resealed key");
        assert_eq!(recovered.private, recovered_again.private);

        let field: SecretField = serde_json::from_value(secret.clone()).expect("secret envelope");
        let nonce: [u8; 24] = B64
            .decode(field.nonce)
            .expect("decode fixture nonce")
            .try_into()
            .expect("24-byte fixture nonce");
        let plaintext = Zeroizing::new(
            master_key
                .decrypt(&EncryptedBlob {
                    nonce,
                    ciphertext: B64
                        .decode(field.ciphertext)
                        .expect("decode fixture ciphertext"),
                    key_version: field.key_version,
                })
                .expect("decrypt fixture plaintext"),
        );
        assert!(plaintext.ends_with(b"\n"));
        assert_eq!(
            plaintext.as_slice(),
            include_bytes!(
                "../../../oceaniam-crypto/tests/fixtures/legacy-rust-crypto/private-key.pkcs8.pem"
            )
        );
        RsaPrivateKey::from_pkcs8_pem(&plaintext).expect("fixture plaintext is PKCS#8 PEM");

        let wrong_master_key =
            MasterKey::from_hex("1123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .expect("different test master key");
        assert!(RsaKey::from_raw_key(raw.clone(), &wrong_master_key).is_err());

        let mut tampered_secret = secret;
        let ciphertext = tampered_secret["ciphertext"]
            .as_str()
            .expect("fixture ciphertext")
            .to_owned();
        let replacement = if ciphertext.starts_with('A') {
            "B"
        } else {
            "A"
        };
        tampered_secret["ciphertext"] = Value::String(format!("{replacement}{}", &ciphertext[1..]));
        assert!(
            RsaKey::from_raw_key(
                RawKey {
                    secret: tampered_secret,
                    ..raw
                },
                &master_key,
            )
            .is_err()
        );
    }

    // NOTE: AI-generated test
    #[test]
    fn aws_lc_generated_default_key_preserves_pem_storage_contract() {
        use base64::{Engine as _, engine::general_purpose::STANDARD as B64};

        let master_key =
            MasterKey::from_hex("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .expect("test master key");
        let original = RsaKey::new(
            Uuid::now_v7(),
            KeyAlg::try_from(Algorithm::PS512).expect("supported algorithm"),
        );
        assert_eq!(original.private.bit_size(), 4096);
        let expected_public_der = original.private.public_key_der();
        let raw = original
            .into_raw_key(&master_key)
            .expect("seal generated AWS-LC key");
        let field: SecretField =
            serde_json::from_value(raw.secret.clone()).expect("secret envelope");
        let nonce: [u8; 24] = B64
            .decode(field.nonce)
            .expect("decode nonce")
            .try_into()
            .expect("24-byte nonce");
        let plaintext = Zeroizing::new(
            master_key
                .decrypt(&EncryptedBlob {
                    nonce,
                    ciphertext: B64.decode(field.ciphertext).expect("decode ciphertext"),
                    key_version: field.key_version,
                })
                .expect("decrypt generated key"),
        );
        let pem = std::str::from_utf8(&plaintext).expect("generated PEM is UTF-8");
        assert!(pem.starts_with("-----BEGIN PRIVATE KEY-----\n"));
        assert!(pem.ends_with("-----END PRIVATE KEY-----\n"));
        assert!(!pem.contains('\r'));
        for line in pem
            .lines()
            .skip(1)
            .take_while(|line| !line.starts_with("-----END"))
        {
            assert!(!line.is_empty());
            assert!(line.len() <= 64);
        }

        let recovered =
            RsaKey::from_raw_key(raw, &master_key).expect("reload generated AWS-LC key");
        assert_eq!(recovered.private.public_key_der(), expected_public_der);
    }

    // NOTE: AI-generated test
    #[tokio::test]
    async fn unsupported_rsa_generation_size_fails_without_fallback() {
        let algorithm = KeyAlg::try_from(Algorithm::RS256).expect("supported algorithm");
        let synchronous = RsaKey::with_bit_size(Uuid::now_v7(), algorithm.clone(), 1024)
            .expect_err("1024-bit generation must be rejected");
        assert!(matches!(
            synchronous,
            Error::Rsa { source, .. } if source.is_unsupported_key_size()
        ));

        let asynchronous = RsaKey::generate_with_bit_size(Uuid::now_v7(), algorithm, 2056)
            .await
            .expect_err("non-AWS-LC generation size must be rejected");
        assert!(matches!(
            asynchronous,
            Error::Rsa { source, .. } if source.is_unsupported_key_size()
        ));
    }
}
