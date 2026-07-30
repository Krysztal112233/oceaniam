use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use chacha20poly1305::{AeadCore, KeyInit, XChaCha20Poly1305, XNonce, aead::Aead};
use oceaniam_common::run_cpu_bound;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use totp_rs::{TOTP, qrcodegen_image::image::EncodableLayout};
use tracing::field::Empty;

use crate::error::Error;

#[derive(Debug)]
pub struct Password(String);

impl Password {
    pub(crate) async fn with_password(
        password: String,
        argon2: Argon2<'static>,
    ) -> Result<Password, Error> {
        let queue_span = tracing::info_span!(
            "credentials.argon2.queue",
            otel.kind = "internal",
            cpu.operation = "argon2.hash"
        );

        run_cpu_bound(queue_span, move |parent| {
            let params = argon2.params();
            let span = tracing::info_span!(
                parent: &parent,
                "credentials.argon2.hash",
                otel.kind = "internal",
                otel.status_code = Empty,
                otel.status_description = Empty,
                argon2.memory_cost_kib = params.m_cost(),
                argon2.time_cost = params.t_cost(),
                argon2.parallelism = params.p_cost(),
            );
            let result = span.in_scope(|| {
                let salt = SaltString::generate(&mut OsRng);
                argon2
                    .hash_password(password.as_bytes(), &salt)
                    .map(|hash| Password(hash.to_string()))
                    .map_err(Error::from)
            });
            if result.is_err() {
                span.record("otel.status_code", "ERROR");
                span.record("otel.status_description", "argon2 hash failed");
            }
            result
        })
        .await?
    }

    pub(crate) fn from_phc(phc: impl Into<String>) -> Self {
        Self(phc.into())
    }

    pub async fn verify(&self, password: &str) -> Result<bool, Error> {
        let password = password.to_owned();
        let phc = self.0.clone();
        let queue_span = tracing::info_span!(
            "credentials.argon2.queue",
            otel.kind = "internal",
            cpu.operation = "argon2.verify"
        );

        run_cpu_bound(queue_span, move |parent| {
            let span = tracing::info_span!(
                parent: &parent,
                "credentials.argon2.verify",
                otel.kind = "internal",
                otel.status_code = Empty,
                otel.status_description = Empty,
                argon2.memory_cost_kib = Empty,
                argon2.time_cost = Empty,
                argon2.parallelism = Empty,
            );
            let result = span.in_scope(|| {
                let password_hash = PasswordHash::new(&phc)?;
                if let Some(value) = password_hash.params.get_decimal("m") {
                    span.record("argon2.memory_cost_kib", value);
                }
                if let Some(value) = password_hash.params.get_decimal("t") {
                    span.record("argon2.time_cost", value);
                }
                if let Some(value) = password_hash.params.get_decimal("p") {
                    span.record("argon2.parallelism", value);
                }

                match Argon2::default().verify_password(password.as_bytes(), &password_hash) {
                    Ok(()) => Ok(true),
                    Err(argon2::password_hash::Error::Password) => Err(Error::Password {
                        source: argon2::password_hash::Error::Password,
                        location: snafu::location!(),
                    }),
                    Err(_) => Ok(false),
                }
            });
            if result.is_err() {
                span.record("otel.status_code", "ERROR");
                span.record("otel.status_description", "argon2 verification failed");
            }
            result
        })
        .await?
    }

    pub fn into_phc(self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct EncryptedTotp(pub(crate) String);

impl EncryptedTotp {
    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
pub struct Totp(totp_rs::TOTP);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TotpVerifyResult {
    pub success: bool,
    pub matched_step: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TotpStorage {
    nonce: String,
    payload: String,
}

impl Totp {
    #[allow(dead_code)]
    pub(crate) fn new(totp: totp_rs::TOTP) -> Self {
        Self(totp)
    }

    #[tracing::instrument(
        level = "info",
        name = "credentials.totp.generate",
        skip_all,
        fields(otel.kind = "internal")
    )]
    pub fn generate(issuer: &str, account_name: &str) -> Result<Self, Error> {
        let totp = TOTP {
            issuer: Some(issuer.to_string()),
            account_name: account_name.to_string(),
            ..Default::default()
        };

        Ok(Self(totp))
    }

    pub fn provisioning_uri(&self) -> String {
        self.0.get_url()
    }

    #[tracing::instrument(
        level = "info",
        name = "credentials.totp.decrypt",
        skip_all,
        fields(otel.kind = "internal")
    )]
    pub fn from_encrypted(base64_string: &str, key: &[u8]) -> Result<Self, Error> {
        let TotpStorage { nonce, payload } = {
            let decoded = STANDARD.decode(base64_string)?;
            serde_json::from_slice(&decoded)?
        };

        let nonce = { STANDARD.decode(nonce)? };

        // Well...WTF.
        #[allow(deprecated)]
        let totp = {
            let cipher: XChaCha20Poly1305 = XChaCha20Poly1305::new_from_slice(key)?;
            let ciphertext = STANDARD.decode(payload)?;
            let totp_payload_decrypted =
                cipher.decrypt(XNonce::from_slice(&nonce), ciphertext.as_bytes())?;

            serde_json::from_slice(&totp_payload_decrypted)?
        };

        Ok(Self(totp))
    }

    #[tracing::instrument(
        level = "info",
        name = "credentials.totp.encrypt",
        skip_all,
        fields(otel.kind = "internal")
    )]
    pub fn to_encrypted(self, key: &[u8]) -> Result<EncryptedTotp, Error> {
        let nonce = XChaCha20Poly1305::generate_nonce(OsRng);
        let payload = {
            let cipher: XChaCha20Poly1305 = XChaCha20Poly1305::new_from_slice(key)?;

            let plaintext = serde_json::to_string(&self.0)?;
            let ciphertext = cipher.encrypt(&nonce, plaintext.as_bytes())?;

            STANDARD.encode(ciphertext)
        };

        let totp_storage = TotpStorage {
            nonce: STANDARD.encode(nonce),
            payload,
        };

        Ok(EncryptedTotp(
            STANDARD.encode(serde_json::to_vec(&totp_storage)?),
        ))
    }

    /// Verifies the provided TOTP token within the current skew window.
    ///
    /// Returns whether verification succeeded and, when it did, which time step
    /// matched the provided token.
    #[tracing::instrument(
        level = "info",
        name = "credentials.totp.verify",
        skip_all,
        fields(otel.kind = "internal")
    )]
    pub fn verify(&self, token: &str) -> Result<TotpVerifyResult, Error> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let current_step = now / self.0.step;
        let base_step = current_step.saturating_sub(self.0.skew as u64);
        let window = (self.0.skew as u64) * 2 + 1;

        for offset in 0..window {
            let step = base_step + offset;
            if self.0.generate(step * self.0.step) == token {
                return Ok(TotpVerifyResult {
                    success: true,
                    matched_step: Some(step),
                });
            }
        }

        Ok(TotpVerifyResult {
            success: false,
            matched_step: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::{self, Write};
    use std::sync::{Arc, Mutex};

    use argon2::{Algorithm, Params, Version};
    use tracing::Instrument as _;
    use tracing_subscriber::fmt::format::FmtSpan;

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

    const TEST_KEY: &[u8] = b"0123456789abcdef0123456789abcdef";

    #[inline(always)]
    fn test_totp() -> totp_rs::TOTP {
        totp_rs::TOTP::default()
    }

    // NOTE: AI-generated test
    #[test]
    fn totp_encryption_round_trip_preserves_configuration() {
        let original = test_totp();

        let encrypted = Totp(original.clone())
            .to_encrypted(TEST_KEY)
            .expect("encryption should succeed");
        let decrypted =
            Totp::from_encrypted(encrypted.as_str(), TEST_KEY).expect("decryption should work");

        assert_eq!(decrypted.0, original);
    }

    // NOTE: AI-generated test
    #[test]
    fn totp_decryption_fails_with_wrong_key() {
        let encrypted = Totp(test_totp())
            .to_encrypted(TEST_KEY)
            .expect("encryption should succeed");

        let result = Totp::from_encrypted(encrypted.as_str(), b"fedcba9876543210fedcba9876543210");

        assert!(matches!(result, Err(Error::Aead { .. })));
    }

    // NOTE: AI-generated test
    #[test]
    fn totp_verify_returns_success_and_matched_step_for_current_token() {
        let inner = test_totp();
        let token = inner
            .generate_current()
            .expect("current TOTP token should be generated");
        let current_step = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_secs()
            / inner.step;

        let result = Totp::new(inner)
            .verify(&token)
            .expect("verification should succeed");

        assert!(result.success);
        assert_eq!(result.matched_step, Some(current_step));
    }

    // NOTE: AI-generated test
    #[test]
    fn totp_generate_creates_valid_totp() {
        let totp =
            Totp::generate("TestApp", "user@test.com").expect("TOTP generation should succeed");
        let uri = totp.provisioning_uri();
        assert!(uri.starts_with("otpauth://"));
        assert!(uri.contains("TestApp"));
        assert!(uri.contains("user%40test.com"));
    }

    // NOTE: AI-generated test
    #[test]
    fn totp_generated_token_verifies() {
        let totp =
            Totp::generate("TestApp", "user@test.com").expect("TOTP generation should succeed");
        let token = totp
            .0
            .generate_current()
            .expect("current TOTP token should be generated");
        let result = totp.verify(&token).expect("verification should succeed");
        assert!(result.success);
    }

    // NOTE: AI-generated test
    #[tokio::test]
    async fn argon2_spans_cross_blocking_dispatch_without_secrets() {
        let bytes = Arc::new(Mutex::new(Vec::new()));
        let writer = bytes.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(move || Buffer(writer.clone()))
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .finish();
        let dispatch = tracing::Dispatch::new(subscriber);
        let _default = tracing::dispatcher::set_default(&dispatch);

        let password = "test-password-must-not-appear".to_owned();
        let params = Params::new(8, 1, 1, Some(16)).expect("low-cost test params");
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let phc = async {
            Password::with_password(password.clone(), argon2)
                .await
                .expect("hash password")
        }
        .instrument(tracing::info_span!("test.request"))
        .await;
        assert!(phc.verify(&password).await.expect("verify password"));
        let phc = phc.into_phc();

        let output = String::from_utf8(bytes.lock().expect("trace buffer lock").clone())
            .expect("utf8 trace output");
        assert!(output.contains("credentials.argon2.queue"));
        assert!(output.contains("credentials.argon2.hash"));
        assert!(output.contains("credentials.argon2.verify"));
        assert!(output.contains("test.request"));
        assert!(!output.contains(&password));
        assert!(!output.contains(&phc));
    }
}
