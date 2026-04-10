use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use oceaniam_common::consts;

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
