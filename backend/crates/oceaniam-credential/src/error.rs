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

oceaniam_common::located_from!(sea_orm::error::DbErr => Error::Db);

oceaniam_common::located_from!(password_hash::Error => Error::Password);

oceaniam_common::located_from!(tokio::task::JoinError => Error::Join);

oceaniam_common::located_from!(base64::DecodeError => Error::Base64);

oceaniam_common::located_from!(serde_json::Error => Error::SerdeJson);

oceaniam_common::located_from!(totp_rs::TotpUrlError => Error::Totp);

oceaniam_common::located_from!(SystemTimeError => Error::SystemTime);

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

impl From<oceaniam_common::crypto::CryptoError> for Error {
    fn from(source: oceaniam_common::crypto::CryptoError) -> Self {
        use oceaniam_common::crypto::CryptoError;
        match source {
            CryptoError::AuthenticationFailed { location }
            | CryptoError::Encryption { location } => Error::Aead { location },
            CryptoError::EmptyKey { location }
            | CryptoError::InsecureKey { location }
            | CryptoError::InvalidKeyLength { location }
            | CryptoError::MissingEnvVar { location, .. } => Error::InvalidLength { location },
            CryptoError::HexDecode { location, .. } => Error::InvalidLength { location },
            CryptoError::Base64 { source, location } => Error::Base64 { source, location },
        }
    }
}
