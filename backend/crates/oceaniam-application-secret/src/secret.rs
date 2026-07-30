use std::fmt;

use rand::distributions::{Alphanumeric, DistString};
use zeroize::Zeroizing;

use crate::ApplicationSecretError;

pub const APPLICATION_SECRET_PREFIX: &str = "app_";
pub const APPLICATION_SECRET_RANDOM_LENGTH: usize = 32;
pub const APPLICATION_SECRET_LENGTH: usize =
    APPLICATION_SECRET_PREFIX.len() + APPLICATION_SECRET_RANDOM_LENGTH;
pub const STORED_PREFIX_LENGTH: usize = APPLICATION_SECRET_PREFIX.len() + 8;
pub const VISIBLE_PREFIX_LENGTH: usize = 8;

pub struct ApplicationSecret(Zeroizing<String>);

impl ApplicationSecret {
    #[tracing::instrument(
        level = "info",
        name = "application_secret.generate",
        skip_all,
        fields(otel.kind = "internal")
    )]
    pub fn generate() -> Self {
        let random = Zeroizing::new(
            Alphanumeric.sample_string(&mut rand::thread_rng(), APPLICATION_SECRET_RANDOM_LENGTH),
        );
        Self(Zeroizing::new(format!(
            "{APPLICATION_SECRET_PREFIX}{}",
            random.as_str()
        )))
    }

    pub fn parse_owned(secret: String) -> Result<Self, ApplicationSecretError> {
        validate_application_secret(&secret)?;
        Ok(Self(Zeroizing::new(secret)))
    }

    pub fn expose(&self) -> &str {
        self.0.as_str()
    }

    pub fn stored_prefix(&self) -> &str {
        &self.0[..STORED_PREFIX_LENGTH]
    }

    pub fn masked(&self) -> String {
        format!("{}...", &self.0[..VISIBLE_PREFIX_LENGTH])
    }
}

impl fmt::Debug for ApplicationSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ApplicationSecret([REDACTED])")
    }
}

pub fn validate_application_secret(secret: &str) -> Result<(), ApplicationSecretError> {
    let random = secret
        .strip_prefix(APPLICATION_SECRET_PREFIX)
        .ok_or(ApplicationSecretError::InvalidSecretFormat)?;

    if secret.len() != APPLICATION_SECRET_LENGTH
        || random.len() != APPLICATION_SECRET_RANDOM_LENGTH
        || !random.bytes().all(|byte| byte.is_ascii_alphanumeric())
    {
        return Err(ApplicationSecretError::InvalidSecretFormat);
    }

    Ok(())
}

pub fn stored_prefix(secret: &str) -> Result<&str, ApplicationSecretError> {
    validate_application_secret(secret)?;
    Ok(&secret[..STORED_PREFIX_LENGTH])
}

pub fn masked_from_stored_prefix(prefix: &str) -> Result<String, ApplicationSecretError> {
    if prefix.len() != STORED_PREFIX_LENGTH
        || !prefix.starts_with(APPLICATION_SECRET_PREFIX)
        || !prefix
            .as_bytes()
            .iter()
            .skip(APPLICATION_SECRET_PREFIX.len())
            .all(u8::is_ascii_alphanumeric)
    {
        return Err(ApplicationSecretError::InvalidSecretFormat);
    }

    Ok(format!("{}...", &prefix[..VISIBLE_PREFIX_LENGTH]))
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: AI-generated test
    #[test]
    fn generated_secret_has_expected_format() {
        let secret = ApplicationSecret::generate();

        assert_eq!(secret.expose().len(), APPLICATION_SECRET_LENGTH);
        assert!(validate_application_secret(secret.expose()).is_ok());
        assert_eq!(secret.stored_prefix().len(), STORED_PREFIX_LENGTH);
    }

    // NOTE: AI-generated test
    #[test]
    fn rejects_invalid_secret_format() {
        assert_eq!(
            validate_application_secret("app_not-long-enough"),
            Err(ApplicationSecretError::InvalidSecretFormat)
        );
        assert_eq!(
            validate_application_secret("app_0123456789012345678901234567890!"),
            Err(ApplicationSecretError::InvalidSecretFormat)
        );
    }

    // NOTE: AI-generated test
    #[test]
    fn stored_prefix_preserves_existing_mask() {
        let secret = "app_01234567890123456789012345678901";
        let prefix = stored_prefix(secret).unwrap();

        assert_eq!(prefix, "app_01234567");
        assert_eq!(masked_from_stored_prefix(prefix).unwrap(), "app_0123...");
    }

    // NOTE: AI-generated test
    #[test]
    fn debug_does_not_reveal_plaintext() {
        let plaintext = "app_01234567890123456789012345678901";
        let secret = ApplicationSecret::parse_owned(plaintext.to_owned()).unwrap();
        let debug = format!("{secret:?}");

        assert!(!debug.contains(plaintext));
        assert!(debug.contains("REDACTED"));
    }
}
