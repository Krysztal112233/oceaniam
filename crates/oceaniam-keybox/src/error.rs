use jsonwebtoken::Algorithm;
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

    #[error("unimplemented jwt alogrithm: {0}")]
    UnimplementedJwtAlogrithm(String),

    #[error("{0}")]
    Pkcs8(#[from] rsa::pkcs8::Error),
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
