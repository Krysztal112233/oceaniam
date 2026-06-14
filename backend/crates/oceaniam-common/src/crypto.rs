use chacha20poly1305::{AeadCore, KeyInit, XChaCha20Poly1305, aead::Aead};
use snafu::{Location, Snafu};
use zeroize::Zeroizing;

/// Errors from the master-key / AEAD layer.
#[derive(Debug, Snafu)]
pub enum CryptoError {
    #[snafu(display("invalid key length at {location}"))]
    InvalidKeyLength { location: Location },

    #[snafu(display("hex decode error: {source} at {location}"))]
    HexDecode {
        source: hex::FromHexError,
        location: Location,
    },

    #[snafu(display("AEAD encryption failed at {location}"))]
    Encryption { location: Location },

    /// Poly1305 tag mismatch OR wrong KEK — intentionally indistinguishable.
    #[snafu(display("authentication failed (wrong key or tampered data) at {location}"))]
    AuthenticationFailed { location: Location },

    #[snafu(display("base64 decode error: {source} at {location}"))]
    Base64 {
        source: base64::DecodeError,
        location: Location,
    },
}

impl From<hex::FromHexError> for CryptoError {
    fn from(source: hex::FromHexError) -> Self {
        Self::HexDecode {
            source,
            location: snafu::location!(),
        }
    }
}

impl From<base64::DecodeError> for CryptoError {
    fn from(source: base64::DecodeError) -> Self {
        Self::Base64 {
            source,
            location: snafu::location!(),
        }
    }
}

impl From<crypto_common::InvalidLength> for CryptoError {
    fn from(_: crypto_common::InvalidLength) -> Self {
        Self::InvalidKeyLength {
            location: snafu::location!(),
        }
    }
}

/// An encrypted blob: random nonce + ciphertext + the KEK version that produced it.
#[derive(Debug, Clone)]
pub struct EncryptedBlob {
    pub nonce: [u8; 24],
    pub ciphertext: Vec<u8>,
    pub key_version: u32,
}

/// The Key Encryption Key (KEK). Held in memory for the process lifetime,
/// zeroized on drop.
#[derive(Debug, Clone)]
pub struct MasterKey(Zeroizing<[u8; 32]>);

impl MasterKey {
    /// Parse a 64-char hex string (32 bytes) into a `MasterKey`.
    pub fn from_hex(hex_str: &str) -> Result<Self, CryptoError> {
        let bytes = hex::decode(hex_str)?;
        if bytes.len() != 32 {
            return Err(CryptoError::InvalidKeyLength {
                location: snafu::location!(),
            });
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self(Zeroizing::new(arr)))
    }

    /// Read the KEK from the `OCEANIAM__MASTER_KEY` environment variable.
    pub fn from_env() -> Result<Self, CryptoError> {
        let raw =
            std::env::var("OCEANIAM__MASTER_KEY").map_err(|_| CryptoError::InvalidKeyLength {
                location: snafu::location!(),
            })?;
        Self::from_hex(&raw)
    }

    /// Encrypt `plaintext` with a fresh random nonce. The `key_version` is
    /// stamped onto the blob.
    pub fn encrypt(
        &self,
        plaintext: &[u8],
        key_version: u32,
    ) -> Result<EncryptedBlob, CryptoError> {
        let cipher = XChaCha20Poly1305::new_from_slice(&self.0[..])?;
        let nonce = XChaCha20Poly1305::generate_nonce(&mut rand::thread_rng());
        let ciphertext =
            cipher
                .encrypt(&nonce, plaintext)
                .map_err(|_| CryptoError::Encryption {
                    location: snafu::location!(),
                })?;

        Ok(EncryptedBlob {
            nonce: nonce.into(),
            ciphertext,
            key_version,
        })
    }

    /// Decrypt an `EncryptedBlob`. Any tamper or wrong-KEK surfaces as
    /// `AuthenticationFailed`.
    pub fn decrypt(&self, blob: &EncryptedBlob) -> Result<Vec<u8>, CryptoError> {
        let cipher = XChaCha20Poly1305::new_from_slice(&self.0[..])?;
        cipher
            .decrypt((&blob.nonce).into(), blob.ciphertext.as_ref())
            .map_err(|_| CryptoError::AuthenticationFailed {
                location: snafu::location!(),
            })
    }

    /// Returns the raw key bytes (for callers that need to construct their
    /// own cipher, e.g. the TOTP path which has its own nonce management).
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts;

    fn test_key() -> MasterKey {
        MasterKey::from_hex("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
            .unwrap()
    }

    // NOTE: AI-generated test
    #[test]
    fn from_hex_accepts_valid_64_char_lowercase() {
        assert!(
            MasterKey::from_hex("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .is_ok()
        );
    }

    // NOTE: AI-generated test
    #[test]
    fn from_hex_accepts_valid_64_char_uppercase() {
        assert!(
            MasterKey::from_hex("0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF")
                .is_ok()
        );
    }

    // NOTE: AI-generated test
    #[test]
    fn from_hex_rejects_empty() {
        assert!(MasterKey::from_hex("").is_err());
    }

    // NOTE: AI-generated test
    #[test]
    fn from_hex_rejects_wrong_length_63() {
        assert!(
            MasterKey::from_hex("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcde")
                .is_err()
        );
    }

    // NOTE: AI-generated test
    #[test]
    fn from_hex_rejects_wrong_length_65() {
        assert!(
            MasterKey::from_hex(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0"
            )
            .is_err()
        );
    }

    // NOTE: AI-generated test
    #[test]
    fn from_hex_rejects_non_hex() {
        assert!(
            MasterKey::from_hex("zz3456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .is_err()
        );
    }

    // NOTE: AI-generated test
    #[test]
    fn encrypt_decrypt_round_trip_empty() {
        let key = test_key();
        let blob = key.encrypt(b"", consts::KEK_VERSION_CURRENT).unwrap();
        let plaintext = key.decrypt(&blob).unwrap();
        assert!(plaintext.is_empty());
    }

    // NOTE: AI-generated test
    #[test]
    fn encrypt_decrypt_round_trip_1_byte() {
        let key = test_key();
        let blob = key.encrypt(b"x", consts::KEK_VERSION_CURRENT).unwrap();
        let plaintext = key.decrypt(&blob).unwrap();
        assert_eq!(plaintext, b"x");
    }

    // NOTE: AI-generated test
    #[test]
    fn encrypt_decrypt_round_trip_pem_sized() {
        let key = test_key();
        let data = vec![0x41u8; 2048];
        let blob = key.encrypt(&data, consts::KEK_VERSION_CURRENT).unwrap();
        let plaintext = key.decrypt(&blob).unwrap();
        assert_eq!(plaintext, data);
    }

    // NOTE: AI-generated test
    #[test]
    fn encrypt_decrypt_round_trip_large() {
        let key = test_key();
        let data = vec![0x42u8; 16384];
        let blob = key.encrypt(&data, consts::KEK_VERSION_CURRENT).unwrap();
        let plaintext = key.decrypt(&blob).unwrap();
        assert_eq!(plaintext, data);
    }

    // NOTE: AI-generated test
    #[test]
    fn decrypt_with_wrong_key_returns_authentication_failed() {
        let key1 = test_key();
        let key2 =
            MasterKey::from_hex("fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210")
                .unwrap();

        let blob = key1
            .encrypt(b"secret", consts::KEK_VERSION_CURRENT)
            .unwrap();
        let result = key2.decrypt(&blob);
        assert!(matches!(
            result,
            Err(CryptoError::AuthenticationFailed { .. })
        ));
    }

    // NOTE: AI-generated test
    #[test]
    fn ciphertext_tamper_returns_authentication_failed() {
        let key = test_key();
        let mut blob = key.encrypt(b"secret", consts::KEK_VERSION_CURRENT).unwrap();
        blob.ciphertext[0] ^= 0xFF;
        let result = key.decrypt(&blob);
        assert!(matches!(
            result,
            Err(CryptoError::AuthenticationFailed { .. })
        ));
    }

    // NOTE: AI-generated test
    #[test]
    fn nonce_tamper_returns_authentication_failed() {
        let key = test_key();
        let mut blob = key.encrypt(b"secret", consts::KEK_VERSION_CURRENT).unwrap();
        blob.nonce[0] ^= 0xFF;
        let result = key.decrypt(&blob);
        assert!(matches!(
            result,
            Err(CryptoError::AuthenticationFailed { .. })
        ));
    }

    // NOTE: AI-generated test
    #[test]
    fn key_version_round_trips() {
        let key = test_key();
        let blob = key.encrypt(b"data", 42).unwrap();
        assert_eq!(blob.key_version, 42);
    }

    // NOTE: AI-generated test
    #[test]
    fn from_env_reads_master_key() {
        // SAFETY: test-only env mutation. serial with other from_env tests.
        unsafe {
            std::env::set_var(
                "OCEANIAM__MASTER_KEY",
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            );
        }
        assert!(MasterKey::from_env().is_ok());
    }

    // NOTE: AI-generated test
    #[test]
    fn from_env_missing_returns_err() {
        unsafe {
            std::env::remove_var("OCEANIAM__MASTER_KEY");
        }
        assert!(MasterKey::from_env().is_err());
    }
}
