use oceaniam_crypto::Algorithm;
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
        source: oceaniam_crypto::RsaError,
        location: Location,
    },

    #[snafu(display("{source} at {location}"))]
    Jwt {
        source: oceaniam_crypto::JwtError,
        location: Location,
    },

    #[snafu(display("{source} at {location}"))]
    Json {
        source: serde_json::Error,
        location: Location,
    },

    #[snafu(display("unimplemented jwt alogrithm: {alg} at {location}"))]
    UnimplementedJwtAlogrithm { alg: String, location: Location },

    #[snafu(display("key id={id} already exists in keybox at {location}"))]
    KeyAlreadyExists { id: String, location: Location },

    #[snafu(display("key id={id} not found in keybox at {location}"))]
    KeyNotFound { id: Uuid, location: Location },

    #[snafu(display("{msg} at {location}"))]
    Internal { msg: String, location: Location },

    #[snafu(display("{source} at {location}"))]
    Crypto {
        source: oceaniam_common::crypto::CryptoError,
        location: Location,
    },

    #[snafu(display("CPU-bound task join error: {source} at {location}"))]
    Join {
        source: tokio::task::JoinError,
        location: Location,
    },
}

impl Error {
    pub fn unimplemented_jwt_alogrithm(key_alg: Algorithm) -> Self {
        Self::UnimplementedJwtAlogrithm {
            alg: format!("{key_alg:?}"),
            location: snafu::location!(),
        }
    }

    pub(crate) fn is_jwt_provider_initialization(&self) -> bool {
        matches!(
            self,
            Self::Jwt { source, .. } if source.is_provider_initialization()
        )
    }
}

oceaniam_common::located_from!(sea_orm::error::DbErr => Error::Db);

oceaniam_common::located_from!(oceaniam_crypto::RsaError => Error::Rsa);

oceaniam_common::located_from!(oceaniam_crypto::JwtError => Error::Jwt);

oceaniam_common::located_from!(serde_json::Error => Error::Json);

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

oceaniam_common::located_from!(tokio::task::JoinError => Error::Join);

oceaniam_common::located_from!(oceaniam_common::crypto::CryptoError => Error::Crypto);
