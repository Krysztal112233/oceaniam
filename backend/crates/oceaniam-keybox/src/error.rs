use jsonwebtoken::Algorithm;
use oceaniam_database::model::sea_orm_active_enums::KeyAlg;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Db(#[from] sea_orm::error::DbErr),

    #[error("mismatced key algorithm: {0}")]
    MismatchedKeyAlg(KeyAlg),

    #[error("{0}")]
    Rsa(#[from] rsa::Error),

    #[error("{0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("{0}")]
    Json(#[from] serde_json::Error),

    #[error("unimplemented jwt alogrithm: {0}")]
    UnimplementedJwtAlogrithm(String),

    #[error("{0}")]
    Pkcs8(#[from] rsa::pkcs8::Error),

    #[error("{0}")]
    Pkcs1(#[from] rsa::pkcs1::Error),

    #[error("key id={0} already exists in keybox")]
    KeyAlreadyExists(String),

    #[error("key id={0} not found in keybox")]
    KeyNotFound(Uuid),

    #[error("{0}")]
    Internal(String),
}

impl Error {
    pub fn unimplemented_jwt_alogrithm(key_alg: Algorithm) -> Self {
        Self::UnimplementedJwtAlogrithm(
            match key_alg {
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
        )
    }
}

impl From<oceaniam_database::Error> for Error {
    fn from(e: oceaniam_database::Error) -> Self {
        match e {
            oceaniam_database::Error::Db(db_err) => Self::Db(db_err),
            oceaniam_database::Error::Json(e) => Self::Json(e),
            oceaniam_database::Error::CustomMessage(_, msg) => Self::Internal(msg),
        }
    }
}
