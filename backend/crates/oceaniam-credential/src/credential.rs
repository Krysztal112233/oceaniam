use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use chacha20poly1305::{AeadCore, KeyInit, XChaCha20Poly1305, XNonce, aead::Aead};
use oceaniam_common::consts;
use serde::{Deserialize, Serialize};
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
                Err(argon2::password_hash::Error::Password) => {
                    Err(Error::Password(argon2::password_hash::Error::Password))
                }
                Err(_) => Ok(false),
            }
        })
        .await?
    }

    pub fn into_phc(self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug)]
pub struct Totp(totp_rs::TOTP);

#[derive(Debug, Serialize, Deserialize)]
pub struct TotpStorage {
    nonce: String,
    payload: String,
}

impl Totp {
    pub(crate) fn new(totp: totp_rs::TOTP) -> Self {
        Self(totp)
    }

    pub fn from_encrypted(base64_string: String, key: &str) -> Result<Self, Error> {
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

    pub fn to_encrypted(self, key: &str) -> Result<String, Error> {
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

        Ok(STANDARD.encode(serde_json::to_vec(&totp_storage)?))
    }

    pub fn verify(&self, token: &str) -> Result<bool, Error> {
        Ok(self.0.check_current(token)?)
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
        let decrypted = Totp::from_encrypted(encrypted, TEST_KEY).expect("decryption should work");

        assert_eq!(decrypted.0, original);
    }

    // NOTE: AI-generated test
    #[test]
    fn totp_decryption_fails_with_wrong_key() {
        let encrypted = Totp(test_totp())
            .to_encrypted(TEST_KEY)
            .expect("encryption should succeed");

        let result = Totp::from_encrypted(encrypted, "fedcba9876543210fedcba9876543210");

        assert!(matches!(result, Err(Error::Aead)));
    }
}
