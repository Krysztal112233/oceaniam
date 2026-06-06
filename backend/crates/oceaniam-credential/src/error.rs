use argon2::password_hash;
use chacha20poly1305::aead;
use snafu::{Location, Snafu};
use std::time::SystemTimeError;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{source} at {location}"))]
    Db {
        source: sea_orm::error::DbErr,
        location: Location,
    },

    #[snafu(display("{source} at {location}"))]
    Password {
        source: password_hash::Error,
        location: Location,
    },

    #[snafu(display("Task join error: {source} at {location}"))]
    Join {
        source: tokio::task::JoinError,
        location: Location,
    },

    #[snafu(display("invalid length of creating chipher for XChaCha20Poly1305 at {location}"))]
    InvalidLength { location: Location },

    #[snafu(display("base64 decode error: {source} at {location}"))]
    Base64 {
        source: base64::DecodeError,
        location: Location,
    },

    #[snafu(display("serde json error: {source} at {location}"))]
    SerdeJson {
        source: serde_json::Error,
        location: Location,
    },

    #[snafu(display("aead error at {location}"))]
    Aead { location: Location },

    #[snafu(display("invalid totp algorithm at {location}"))]
    InvalidAlgorithm { location: Location },

    #[snafu(display("totp error: {source} at {location}"))]
    Totp {
        source: totp_rs::TotpUrlError,
        location: Location,
    },

    #[snafu(display("system time error: {source} at {location}"))]
    SystemTime {
        source: SystemTimeError,
        location: Location,
    },
}

impl From<sea_orm::error::DbErr> for Error {
    fn from(source: sea_orm::error::DbErr) -> Self {
        Error::Db {
            source,
            location: snafu::location!(),
        }
    }
}

impl From<password_hash::Error> for Error {
    fn from(source: password_hash::Error) -> Self {
        Error::Password {
            source,
            location: snafu::location!(),
        }
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(source: tokio::task::JoinError) -> Self {
        Error::Join {
            source,
            location: snafu::location!(),
        }
    }
}

impl From<base64::DecodeError> for Error {
    fn from(source: base64::DecodeError) -> Self {
        Error::Base64 {
            source,
            location: snafu::location!(),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(source: serde_json::Error) -> Self {
        Error::SerdeJson {
            source,
            location: snafu::location!(),
        }
    }
}

impl From<totp_rs::TotpUrlError> for Error {
    fn from(source: totp_rs::TotpUrlError) -> Self {
        Error::Totp {
            source,
            location: snafu::location!(),
        }
    }
}

impl From<SystemTimeError> for Error {
    fn from(source: SystemTimeError) -> Self {
        Error::SystemTime {
            source,
            location: snafu::location!(),
        }
    }
}

impl From<crypto_common::InvalidLength> for Error {
    fn from(_: crypto_common::InvalidLength) -> Self {
        Error::InvalidLength {
            location: snafu::location!(),
        }
    }
}

impl From<aead::Error> for Error {
    fn from(_: aead::Error) -> Self {
        Error::Aead {
            location: snafu::location!(),
        }
    }
}

impl From<oceaniam_database::Error> for Error {
    fn from(source: oceaniam_database::Error) -> Self {
        match source {
            oceaniam_database::Error::Db { source, location } => Error::Db { source, location },
            oceaniam_database::Error::Json { source, location } => {
                Error::SerdeJson { source, location }
            }
            oceaniam_database::Error::CustomMessage { msg, location, .. } => Error::Db {
                source: sea_orm::DbErr::Custom(msg),
                location,
            },
        }
    }
}
