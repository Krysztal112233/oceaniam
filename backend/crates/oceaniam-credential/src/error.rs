use argon2::password_hash;
use chacha20poly1305::aead;
use std::time::SystemTimeError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Db(#[from] sea_orm::error::DbErr),

    #[error("{0}")]
    Password(#[from] password_hash::Error),

    #[error("Task join error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("invalid length of creating chipher for XChaCha20Poly1305")]
    InvalidLength,

    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("aead error")]
    Aead,

    #[error("invalid totp algorithm")]
    InvalidAlgorithm,

    #[error("totp error: {0}")]
    Totp(#[from] totp_rs::TotpUrlError),

    #[error("system time error: {0}")]
    SystemTime(#[from] SystemTimeError),
}

impl From<crypto_common::InvalidLength> for Error {
    fn from(_: crypto_common::InvalidLength) -> Self {
        Self::InvalidLength
    }
}

impl From<aead::Error> for Error {
    fn from(_: aead::Error) -> Self {
        Self::Aead
    }
}
