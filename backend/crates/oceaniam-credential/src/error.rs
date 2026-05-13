use argon2::password_hash;
use chacha20poly1305::aead;
use snafu::Snafu;
use std::time::SystemTimeError;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{source}"), context(false))]
    Db { source: sea_orm::error::DbErr },

    #[snafu(display("{source}"), context(false))]
    Password { source: password_hash::Error },

    #[snafu(display("Task join error: {source}"), context(false))]
    Join { source: tokio::task::JoinError },

    #[snafu(display("invalid length of creating chipher for XChaCha20Poly1305"))]
    InvalidLength,

    #[snafu(display("base64 decode error: {source}"), context(false))]
    Base64 { source: base64::DecodeError },

    #[snafu(display("serde json error: {source}"), context(false))]
    SerdeJson { source: serde_json::Error },

    #[snafu(display("aead error"))]
    Aead,

    #[snafu(display("invalid totp algorithm"))]
    InvalidAlgorithm,

    #[snafu(display("totp error: {source}"), context(false))]
    Totp { source: totp_rs::TotpUrlError },

    #[snafu(display("system time error: {source}"), context(false))]
    SystemTime { source: SystemTimeError },
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
