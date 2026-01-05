use argon2::{Argon2, PasswordHash, PasswordVerifier};

use crate::{CredentialVault, error::Error};

#[derive(Debug)]
pub struct Password<'a>(PasswordHash<'a>);

impl<'a> Password<'a> {
    pub fn new(phc: &'a impl AsRef<str>) -> Result<Self, Error> {
        Ok(Self(PasswordHash::new(phc.as_ref())?))
    }

    pub fn verify(&self, password: impl Into<String>) -> Result<bool, Error> {
        match Argon2::default().verify_password(password.into().as_bytes(), &self.0) {
            Ok(()) => Ok(true),
            Err(e) => match e {
                argon2::password_hash::Error::Password => Err(e.into()),
                _ => Ok(false),
            },
        }
    }
}

impl<'a> TryFrom<&'a CredentialVault> for Password<'a> {
    type Error = Error;

    fn try_from(value: &'a CredentialVault) -> Result<Self, Self::Error> {
        Self::new(&value.phc)
    }
}
