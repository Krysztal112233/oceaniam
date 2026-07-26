use std::{collections::BTreeMap, fmt};

use hmac::{Hmac, Mac};
use serde::{Deserialize, Deserializer, de::Error as _};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

use crate::{ApplicationSecretError, validate_application_secret};

const HMAC_DOMAIN_SEPARATOR: &[u8] = b"oceaniam/application-secret/v1\0";
const HMAC_KEY_LENGTH: usize = 32;
pub const APPLICATION_SECRET_VERIFIER_LENGTH: usize = 32;

pub struct ApplicationSecretHmacKey(Zeroizing<[u8; HMAC_KEY_LENGTH]>);

impl ApplicationSecretHmacKey {
    pub fn from_hex_owned(hex: String) -> Result<Self, ApplicationSecretError> {
        let hex = Zeroizing::new(hex);
        if hex.len() != HMAC_KEY_LENGTH * 2 {
            return Err(ApplicationSecretError::InvalidKeyLength);
        }

        let mut key = Zeroizing::new([0u8; HMAC_KEY_LENGTH]);
        hex::decode_to_slice(hex.as_bytes(), key.as_mut())
            .map_err(|_| ApplicationSecretError::InvalidKeyEncoding)?;

        Self::from_zeroizing_bytes(key)
    }

    pub fn from_bytes(key: [u8; HMAC_KEY_LENGTH]) -> Result<Self, ApplicationSecretError> {
        Self::from_zeroizing_bytes(Zeroizing::new(key))
    }

    fn from_zeroizing_bytes(
        key: Zeroizing<[u8; HMAC_KEY_LENGTH]>,
    ) -> Result<Self, ApplicationSecretError> {
        if key.iter().all(|byte| *byte == 0) {
            return Err(ApplicationSecretError::AllZeroKey);
        }
        Ok(Self(key))
    }

    fn as_bytes(&self) -> &[u8; HMAC_KEY_LENGTH] {
        &self.0
    }
}

impl fmt::Debug for ApplicationSecretHmacKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ApplicationSecretHmacKey([REDACTED])")
    }
}

pub struct ApplicationSecretKeyring {
    current_version: i32,
    keys: BTreeMap<i32, ApplicationSecretHmacKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionedVerifier {
    pub hmac_key_version: i32,
    pub verifier: [u8; APPLICATION_SECRET_VERIFIER_LENGTH],
}

impl ApplicationSecretKeyring {
    pub fn new(
        current_version: i32,
        keys: impl IntoIterator<Item = (i32, ApplicationSecretHmacKey)>,
    ) -> Result<Self, ApplicationSecretError> {
        if current_version <= 0 {
            return Err(ApplicationSecretError::InvalidKeyVersion(current_version));
        }

        let mut keyring = BTreeMap::new();
        for (version, key) in keys {
            if version <= 0 {
                return Err(ApplicationSecretError::InvalidKeyVersion(version));
            }
            if keyring.insert(version, key).is_some() {
                return Err(ApplicationSecretError::DuplicateKeyVersion(version));
            }
        }

        if !keyring.contains_key(&current_version) {
            return Err(ApplicationSecretError::MissingCurrentKey(current_version));
        }

        Ok(Self {
            current_version,
            keys: keyring,
        })
    }

    pub fn current_version(&self) -> i32 {
        self.current_version
    }

    pub fn contains_version(&self, version: i32) -> bool {
        self.keys.contains_key(&version)
    }

    pub fn versions(&self) -> impl Iterator<Item = i32> + '_ {
        self.keys.keys().copied()
    }

    pub fn verifier_for_current(
        &self,
        secret: &str,
    ) -> Result<VersionedVerifier, ApplicationSecretError> {
        Ok(VersionedVerifier {
            hmac_key_version: self.current_version,
            verifier: self.verifier(self.current_version, secret)?,
        })
    }

    pub fn verifier(
        &self,
        version: i32,
        secret: &str,
    ) -> Result<[u8; APPLICATION_SECRET_VERIFIER_LENGTH], ApplicationSecretError> {
        validate_application_secret(secret)?;
        let key = self
            .keys
            .get(&version)
            .ok_or(ApplicationSecretError::MissingKeyVersion(version))?;

        let mut mac = Hmac::<Sha256>::new_from_slice(key.as_bytes())
            .expect("32-byte HMAC-SHA-256 key is always valid");
        mac.update(HMAC_DOMAIN_SEPARATOR);
        mac.update(secret.as_bytes());
        Ok(mac.finalize().into_bytes().into())
    }

    pub fn verify(
        &self,
        version: i32,
        secret: &str,
        expected: &[u8],
    ) -> Result<bool, ApplicationSecretError> {
        let expected: &[u8; APPLICATION_SECRET_VERIFIER_LENGTH] = expected
            .try_into()
            .map_err(|_| ApplicationSecretError::InvalidVerifierLength(expected.len()))?;
        let actual = self.verifier(version, secret)?;
        Ok(bool::from(actual.ct_eq(expected)))
    }
}

impl fmt::Debug for ApplicationSecretKeyring {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApplicationSecretKeyring")
            .field("current_version", &self.current_version)
            .field("versions", &self.keys.keys().collect::<Vec<_>>())
            .finish()
    }
}

#[derive(Deserialize)]
struct RawKeyring {
    current_version: i32,
    keys: BTreeMap<String, String>,
}

impl<'de> Deserialize<'de> for ApplicationSecretKeyring {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawKeyring::deserialize(deserializer)?;
        let mut keys = Vec::with_capacity(raw.keys.len());

        for (version, hex) in raw.keys {
            let version = version
                .parse::<i32>()
                .map_err(|_| D::Error::custom(format!("invalid HMAC key version `{version}`")))?;
            let key = ApplicationSecretHmacKey::from_hex_owned(hex).map_err(D::Error::custom)?;
            keys.push((version, key));
        }

        Self::new(raw.current_version, keys).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &str = "app_01234567890123456789012345678901";
    const TEST_KEY_HEX: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

    fn keyring() -> ApplicationSecretKeyring {
        ApplicationSecretKeyring::new(
            1,
            [(
                1,
                ApplicationSecretHmacKey::from_hex_owned(TEST_KEY_HEX.to_owned()).unwrap(),
            )],
        )
        .unwrap()
    }

    // NOTE: AI-generated test
    #[test]
    fn verifier_matches_known_answer() {
        assert_eq!(
            hex::encode(keyring().verifier(1, TEST_SECRET).unwrap()),
            "8ce74261915a0a22b1bf75247495b2dd7b7d3310bab2a7be2627766b797a226f"
        );
    }

    // NOTE: AI-generated test
    #[test]
    fn verifies_correct_secret_and_rejects_wrong_secret() {
        let keyring = keyring();
        let expected = keyring.verifier(1, TEST_SECRET).unwrap();
        let wrong = "app_11234567890123456789012345678901";

        assert!(keyring.verify(1, TEST_SECRET, &expected).unwrap());
        assert!(!keyring.verify(1, wrong, &expected).unwrap());
    }

    // NOTE: AI-generated test
    #[test]
    fn rejects_missing_version_and_invalid_keys() {
        assert_eq!(
            keyring().verifier(2, TEST_SECRET),
            Err(ApplicationSecretError::MissingKeyVersion(2))
        );
        assert_eq!(
            ApplicationSecretHmacKey::from_hex_owned("00".repeat(32)).unwrap_err(),
            ApplicationSecretError::AllZeroKey
        );
        assert_eq!(
            ApplicationSecretHmacKey::from_hex_owned("ab".repeat(31)).unwrap_err(),
            ApplicationSecretError::InvalidKeyLength
        );
    }

    // NOTE: AI-generated test
    #[test]
    fn current_version_can_rotate_while_old_version_still_verifies() {
        let old_key = ApplicationSecretHmacKey::from_hex_owned(TEST_KEY_HEX.to_owned()).unwrap();
        let new_key = ApplicationSecretHmacKey::from_hex_owned("ab".repeat(32)).unwrap();
        let keyring = ApplicationSecretKeyring::new(2, [(1, old_key), (2, new_key)]).unwrap();
        let old_verifier = keyring.verifier(1, TEST_SECRET).unwrap();
        let current = keyring.verifier_for_current(TEST_SECRET).unwrap();

        assert!(keyring.verify(1, TEST_SECRET, &old_verifier).unwrap());
        assert_eq!(current.hmac_key_version, 2);
        assert_ne!(current.verifier, old_verifier);
    }

    // NOTE: AI-generated test
    #[test]
    fn rejects_keyring_without_current_version() {
        let key = ApplicationSecretHmacKey::from_hex_owned(TEST_KEY_HEX.to_owned()).unwrap();
        assert_eq!(
            ApplicationSecretKeyring::new(2, [(1, key)]).unwrap_err(),
            ApplicationSecretError::MissingCurrentKey(2)
        );
    }

    // NOTE: AI-generated test
    #[test]
    fn deserializes_versioned_keyring_without_debug_leakage() {
        let json = format!(r#"{{"current_version":1,"keys":{{"1":"{TEST_KEY_HEX}"}}}}"#);
        let keyring: ApplicationSecretKeyring = serde_json::from_str(&json).unwrap();
        let debug = format!("{keyring:?}");

        assert_eq!(keyring.current_version(), 1);
        assert!(!debug.contains(TEST_KEY_HEX));
        assert!(debug.contains("versions"));
    }
}
