#![doc = include_str!("../README.md")]

use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use serde::{Deserialize, Serialize};

use crate::error::Error;

pub mod credential;
pub mod error;

#[derive(Debug, Deserialize, Serialize)]
pub struct CredentialVault {
    /// This field used to store [PHC](argon2::PasswordHash) string.
    pub phc: String,
}

impl CredentialVault {
    /// User must have at least password login method enabled.
    pub fn new(password: impl AsRef<str>) -> Result<Self, Error> {
        let salt = SaltString::generate(&mut OsRng);

        let password = Argon2::default()
            .hash_password(password.as_ref().as_bytes(), &salt)?
            .to_string();

        Ok(Self { phc: password })
    }
}
