use argon2::Argon2;
use oceaniam_database::{
    helper::{SafeTransactionConnectionTrait, credentials::CredentialsHelper},
    model::{self, prelude::Credentials},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    credential::{EncryptedTotp, Password, Totp, TotpVerifyResult},
    error::Error,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CredentialVault {
    /// This field used to store [PHC](argon2::PasswordHash) string.
    pub phc: String,

    /// This field is optional. Its value is produced by serializing the TOTP struct provided by the
    /// [totp_rs] crate to JSON and encrypting it with [chacha20poly1305::XChaCha20Poly1305].
    ///
    /// It must be decrypted before each use.
    ///
    /// If decryption fails, the TOTP function cannot continue to be used and must be regenerated.
    pub(crate) totp: Option<String>,
}

impl CredentialVault {
    /// User must have at least password login method enabled.
    pub fn with_password(password: impl AsRef<str>, argon2: &Argon2<'_>) -> Result<Self, Error> {
        let phc = Password::with_password(password, argon2)?.into_phc();

        Ok(Self { phc, totp: None })
    }

    pub fn update_password(
        self,
        password: impl AsRef<str>,
        argon2: &Argon2<'_>,
    ) -> Result<Self, Error> {
        let phc = Password::with_password(password, argon2)?.into_phc();

        #[allow(clippy::needless_update)]
        Ok(Self { phc, ..self })
    }

    pub async fn write_to(
        &self,
        id: impl Into<Uuid>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::credentials::Model, Error> {
        let Self { phc, totp } = self;
        let id = id.into();

        Ok(Credentials::upsert_credential(id, phc.clone(), totp.clone(), database).await?)
    }

    pub fn enable_totp(self, encrypted_totp: EncryptedTotp) -> Self {
        Self {
            totp: Some(encrypted_totp.0),
            ..self
        }
    }

    pub fn remove_totp(self) -> Self {
        Self { totp: None, ..self }
    }

    pub fn has_totp(&self) -> bool {
        self.totp.is_some()
    }
}

impl CredentialVault {
    pub async fn verify_password(&self, password: impl AsRef<str>) -> Result<bool, Error> {
        Password::from_phc(self.phc.clone())
            .verify(password.as_ref())
            .await
    }

    pub fn verify_totp(
        &self,
        token: impl AsRef<str>,
        key: &str,
    ) -> Result<TotpVerifyResult, Error> {
        let Some(totp) = self.totp.clone() else {
            return Ok(TotpVerifyResult {
                success: false,
                matched_step: None,
            });
        };

        Totp::from_encrypted(&totp, key)?.verify(token.as_ref())
    }
}

impl From<model::credentials::Model> for CredentialVault {
    fn from(value: model::credentials::Model) -> Self {
        let model::credentials::Model { phc, totp, .. } = value;

        Self { phc, totp }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_KEY: &str = "0123456789abcdef0123456789abcdef";

    // NOTE: AI-generated test
    #[test]
    fn verify_totp_accepts_current_token() {
        let inner = totp_rs::TOTP::default();
        let token = inner
            .generate_current()
            .expect("current TOTP token should be generated");
        let totp = crate::credential::Totp::new(inner)
            .to_encrypted(TEST_KEY)
            .expect("TOTP should be encrypted");
        let vault = CredentialVault {
            phc: "unused".to_string(),
            totp: Some(totp.0),
        };

        let verified = vault
            .verify_totp(&token, TEST_KEY)
            .expect("verification should succeed");

        assert!(verified.success);
        assert!(verified.matched_step.is_some());
    }

    // NOTE: AI-generated test
    #[test]
    fn verify_totp_returns_false_when_totp_is_missing() {
        let vault = CredentialVault {
            phc: "unused".to_string(),
            totp: None,
        };

        let verified = vault
            .verify_totp("123456", TEST_KEY)
            .expect("missing TOTP should not error");

        assert!(!verified.success);
        assert_eq!(verified.matched_step, None);
    }
}
