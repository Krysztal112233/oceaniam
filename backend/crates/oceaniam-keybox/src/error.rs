use jsonwebtoken::Algorithm;
use oceaniam_database::model::sea_orm_active_enums::KeyAlg;
use snafu::Snafu;
use uuid::Uuid;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{source}"), context(false))]
    Db { source: sea_orm::error::DbErr },

    #[snafu(display("mismatced key algorithm: {key_alg}"))]
    MismatchedKeyAlg { key_alg: KeyAlg },

    #[snafu(display("{source}"), context(false))]
    Rsa { source: rsa::Error },

    #[snafu(display("{source}"), context(false))]
    Jwt { source: jsonwebtoken::errors::Error },

    #[snafu(display("{source}"), context(false))]
    Json { source: serde_json::Error },

    #[snafu(display("unimplemented jwt alogrithm: {alg}"))]
    UnimplementedJwtAlogrithm { alg: String },

    #[snafu(display("{source}"), context(false))]
    Pkcs8 { source: rsa::pkcs8::Error },

    #[snafu(display("{source}"), context(false))]
    Pkcs1 { source: rsa::pkcs1::Error },

    #[snafu(display("key id={id} already exists in keybox"))]
    KeyAlreadyExists { id: String },

    #[snafu(display("key id={id} not found in keybox"))]
    KeyNotFound { id: Uuid },

    #[snafu(display("{msg}"))]
    Internal { msg: String },
}

impl Error {
    pub fn unimplemented_jwt_alogrithm(key_alg: Algorithm) -> Self {
        Self::UnimplementedJwtAlogrithm {
            alg: match key_alg {
                Algorithm::HS256 => "HS256",
                Algorithm::HS384 => "HS384",
                Algorithm::HS512 => "HS512",
                Algorithm::ES256 => "ES256",
                Algorithm::ES384 => "ES384",
                Algorithm::RS256 => "RS256",
                Algorithm::RS384 => "RS384",
                Algorithm::RS512 => "RS512",
                Algorithm::PS256 => "PS256",
                Algorithm::PS384 => "PS384",
                Algorithm::PS512 => "PS512",
                Algorithm::EdDSA => "EdDSA",
            }
            .to_string(),
        }
    }
}

impl From<oceaniam_database::Error> for Error {
    fn from(e: oceaniam_database::Error) -> Self {
        match e {
            oceaniam_database::Error::Db { source: db_err } => Self::Db { source: db_err },
            oceaniam_database::Error::Json { source: e } => Self::Json { source: e },
            oceaniam_database::Error::CustomMessage { msg, .. } => Self::Internal { msg },
        }
    }
}
