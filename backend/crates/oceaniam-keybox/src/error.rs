use jsonwebtoken::Algorithm;
use oceaniam_database::model::sea_orm_active_enums::KeyAlg;
use snafu::{Location, Snafu};
use uuid::Uuid;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{source} at {location}"))]
    Db {
        source: sea_orm::error::DbErr,
        location: Location,
    },

    #[snafu(display("mismatced key algorithm: {key_alg} at {location}"))]
    MismatchedKeyAlg { key_alg: KeyAlg, location: Location },

    #[snafu(display("{source} at {location}"))]
    Rsa {
        source: rsa::Error,
        location: Location,
    },

    #[snafu(display("{source} at {location}"))]
    Jwt {
        source: jsonwebtoken::errors::Error,
        location: Location,
    },

    #[snafu(display("{source} at {location}"))]
    Json {
        source: serde_json::Error,
        location: Location,
    },

    #[snafu(display("unimplemented jwt alogrithm: {alg} at {location}"))]
    UnimplementedJwtAlogrithm { alg: String, location: Location },

    #[snafu(display("{source} at {location}"))]
    Pkcs8 {
        source: rsa::pkcs8::Error,
        location: Location,
    },

    #[snafu(display("{source} at {location}"))]
    Pkcs1 {
        source: rsa::pkcs1::Error,
        location: Location,
    },

    #[snafu(display("key id={id} already exists in keybox at {location}"))]
    KeyAlreadyExists { id: String, location: Location },

    #[snafu(display("key id={id} not found in keybox at {location}"))]
    KeyNotFound { id: Uuid, location: Location },

    #[snafu(display("{msg} at {location}"))]
    Internal { msg: String, location: Location },
}

impl Error {
    pub fn unimplemented_jwt_alogrithm(key_alg: Algorithm) -> Self {
        Self::UnimplementedJwtAlogrithm {
            alg: format!("{key_alg:?}"),
            location: snafu::location!(),
        }
    }
}

impl From<sea_orm::error::DbErr> for Error {
    fn from(source: sea_orm::error::DbErr) -> Self {
        Error::Db {
            source,
            location: snafu::location!(),
        }
    }
}

impl From<rsa::Error> for Error {
    fn from(source: rsa::Error) -> Self {
        Error::Rsa {
            source,
            location: snafu::location!(),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(source: jsonwebtoken::errors::Error) -> Self {
        Error::Jwt {
            source,
            location: snafu::location!(),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(source: serde_json::Error) -> Self {
        Error::Json {
            source,
            location: snafu::location!(),
        }
    }
}

impl From<rsa::pkcs8::Error> for Error {
    fn from(source: rsa::pkcs8::Error) -> Self {
        Error::Pkcs8 {
            source,
            location: snafu::location!(),
        }
    }
}

impl From<rsa::pkcs1::Error> for Error {
    fn from(source: rsa::pkcs1::Error) -> Self {
        Error::Pkcs1 {
            source,
            location: snafu::location!(),
        }
    }
}

impl From<oceaniam_database::Error> for Error {
    fn from(e: oceaniam_database::Error) -> Self {
        match e {
            oceaniam_database::Error::Db { source: db_err, .. } => Error::Db {
                source: db_err,
                location: snafu::location!(),
            },
            oceaniam_database::Error::Json { source: e, .. } => Error::Json {
                source: e,
                location: snafu::location!(),
            },
            oceaniam_database::Error::CustomMessage { msg, .. } => Error::Internal {
                msg,
                location: snafu::location!(),
            },
        }
    }
}
