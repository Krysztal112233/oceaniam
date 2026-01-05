use argon2::password_hash;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Password(#[from] password_hash::Error),
}
