use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use chacha20poly1305::{AeadCore, KeyInit, XChaCha20Poly1305, XNonce, aead::Aead};
use oceaniam_common::consts;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use totp_rs::qrcodegen_image::image::EncodableLayout;

use crate::error::Error;

#[derive(Debug)]
pub struct Password(String);

impl Password {
    pub(crate) fn with_password(
        password: impl AsRef<str>,
        argon2: &Argon2<'_>,
    ) -> Result<Password, Error> {
        // Generate salt and hash in one expression to avoid lifetime issues
        let hash_string = {
            let salt = SaltString::generate(&mut OsRng);
            argon2
                .hash_password(password.as_ref().as_bytes(), &salt)?
                .to_string()
        };

        // Leak the string to get a 'static reference (acceptable for password hashes)
        let phc = hash_string;
        Ok(Password(phc))
    }

    pub(crate) fn from_phc(phc: impl Into<String>) -> Self {
        Self(phc.into())
    }

    pub async fn verify(&self, password: &str) -> Result<bool, Error> {
        // Semaphore acquisition stays async; only the CPU-bound hash verification runs on the
        // blocking pool.
        let _permit = consts::MAX_CPU_BOUND_SEMAPHORE
            .acquire()
            .await
            .expect("cpu-bound semaphore should not be closed");

        let password = password.to_owned();
        let phc = self.0.clone();

        tokio::task::spawn_blocking(move || {
            let password_hash = PasswordHash::new(&phc)?;

            match Argon2::default().verify_password(password.as_bytes(), &password_hash) {
                Ok(()) => Ok(true),
                Err(argon2::password_hash::Error::Password) => Err(Error::Password {
                    source: argon2::password_hash::Error::Password,
                    location: snafu::location!(),
                }),
                Err(_) => Ok(false),
            }
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

    pub fn generate(issuer: &str, account_name: &str) -> Result<Self, Error> {
        let mut totp = totp_rs::TOTP::default();
        totp.issuer = Some(issuer.to_string());
        totp.account_name = account_name.to_string();
        Ok(Self(totp))
    }

    pub fn provisioning_uri(&self) -> String {
        self.0.get_url()
    }

    pub fn from_encrypted(base64_string: &str, key: &str) -> Result<Self, Error> {
        let TotpStorage { nonce, payload } = {
            let decoded = STANDARD.decode(base64_string)?;
            serde_json::from_slice(&decoded)?
        };

        let nonce = { STANDARD.decode(nonce)? };

        // Well...WTF.
        #[allow(deprecated)]
        let totp = {
            let cipher: XChaCha20Poly1305 = XChaCha20Poly1305::new_from_slice(key.as_bytes())?;
            let ciphertext = STANDARD.decode(payload)?;
            let totp_payload_decrypted =
                cipher.decrypt(XNonce::from_slice(&nonce), ciphertext.as_bytes())?;

            serde_json::from_slice(&totp_payload_decrypted)?
        };

        Ok(Self(totp))
    }

    pub fn to_encrypted(self, key: &str) -> Result<EncryptedTotp, Error> {
        let nonce = XChaCha20Poly1305::generate_nonce(OsRng);
        let payload = {
            let cipher: XChaCha20Poly1305 = XChaCha20Poly1305::new_from_slice(key.as_bytes())?;

            let plaintext = serde_json::to_string(&self.0)?;
            let ciphertext = cipher.encrypt(&nonce, plaintext.as_bytes())?;

            STANDARD.encode(ciphertext)
        };

        let totp_storage = TotpStorage {
            nonce: STANDARD.encode(nonce.to_vec()),
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

    const TEST_KEY: &str = "0123456789abcdef0123456789abcdef";

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

        let result = Totp::from_encrypted(encrypted.as_str(), "fedcba9876543210fedcba9876543210");

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
}
