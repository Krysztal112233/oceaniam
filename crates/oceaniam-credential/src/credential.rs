use std::sync::LazyLock;

use argon2::{
    Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};

use crate::{CredentialVault, error::Error};

#[derive(Debug)]
pub struct Password(String);

static DEFAULT_ARGON_CFG: LazyLock<Argon2> = LazyLock::new(|| {
    Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        Params::new(
            Params::DEFAULT_M_COST,
            Params::DEFAULT_T_COST,
            16,
            Some(Params::DEFAULT_OUTPUT_LEN),
        )
        .unwrap(),
    )
});

impl Password {
    pub fn with_password(password: impl AsRef<str>) -> Result<Password, Error> {
        // Generate salt and hash in one expression to avoid lifetime issues
        let hash_string = {
            let salt = SaltString::generate(&mut OsRng);
            DEFAULT_ARGON_CFG
                .clone()
                .hash_password(password.as_ref().as_bytes(), &salt)?
                .to_string()
        };

        // Leak the string to get a 'static reference (acceptable for password hashes)
        let phc = hash_string;
        Ok(Password(phc))
    }

    pub fn with_phc(phc: impl Into<String>) -> Self {
        Self(phc.into())
    }

    pub fn verify(&self, password: impl Into<String>) -> Result<bool, Error> {
        let password_hash = PasswordHash::new(&self.0)?;

        match DEFAULT_ARGON_CFG
            .clone()
            .verify_password(password.into().as_bytes(), &password_hash)
        {
            Ok(()) => Ok(true),
            Err(e) => match e {
                argon2::password_hash::Error::Password => Err(e.into()),
                _ => Ok(false),
            },
        }
    }

    pub fn into_phc(self) -> String {
        self.0.to_string()
    }
}

impl From<CredentialVault> for Password {
    fn from(value: CredentialVault) -> Self {
        Self::with_phc(value.phc)
    }
}
