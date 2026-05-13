use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("kid not found: {0}")]
    KidNotFound(String),

    #[error("{0}")]
    Internal(String),
}
