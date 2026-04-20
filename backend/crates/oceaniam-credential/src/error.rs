use argon2::password_hash;
use axum::http::StatusCode;
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

impl From<Error> for oceaniam_common::error::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::Db(db_err) => Self::Db(db_err),
            Error::Password(_) => Self::with_code(
                StatusCode::INTERNAL_SERVER_ERROR,
                oceaniam_common::consts::USER_LOGIN_FAILED_MSG,
            ),
            Error::Join(_) => {
                Self::with_code(StatusCode::INTERNAL_SERVER_ERROR, "task execution failed")
            }
            Error::InvalidLength
            | Error::Base64(_)
            | Error::SerdeJson(_)
            | Error::Aead
            | Error::InvalidAlgorithm => Self::with_code(
                StatusCode::INTERNAL_SERVER_ERROR,
                "invalid totp secret data",
            ),
            Error::Totp(_) => Self::with_code(
                StatusCode::INTERNAL_SERVER_ERROR,
                "invalid totp configuration",
            ),
            Error::SystemTime(_) => Self::with_code(
                StatusCode::INTERNAL_SERVER_ERROR,
                "invalid totp configuration",
            ),
        }
    }
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
