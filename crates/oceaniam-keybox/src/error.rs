use oceaniam_database::model::sea_orm_active_enums::KeyAlg;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("mismatced key algorithm: {0}")]
    MismatchedKeyAlg(KeyAlg),

    #[error("{0}")]
    Rsa(#[from] rsa::Error),

    #[error("{0}")]
    Jwk(#[from] jsonwebtoken::errors::Error),

    #[error("{0}")]
    Serde(#[from] serde_json::Error),
}
