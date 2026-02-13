use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;

use crate::{ApiResponse, ErrorResponse};

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Db(#[from] sea_orm::error::DbErr),

    #[error("{0}")]
    Conf(#[from] config::ConfigError),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("{0}")]
    Internal(String),

    #[error("status: {0}, msg: {1}")]
    CustomMessage(u16, String),
}

impl Error {
    pub fn with_code(code: impl Into<u16>, msg: impl Into<String>) -> Self {
        Self::CustomMessage(code.into(), msg.into())
    }
}

#[allow(clippy::match_single_binding)]
impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Self::CustomMessage(code, _) => StatusCode::from_u16(code).unwrap(),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (
            status,
            ApiResponse::new(ErrorResponse::new(status.to_string())),
        )
            .into_response()
    }
}

#[allow(unused)]
impl From<rsa::pkcs1::Error> for Error {
    fn from(value: rsa::pkcs1::Error) -> Self {
        Self::Jwt(jsonwebtoken::errors::new_error(
            jsonwebtoken::errors::ErrorKind::InvalidKeyFormat,
        ))
    }
}

impl From<rsa::pkcs8::Error> for Error {
    fn from(_: rsa::pkcs8::Error) -> Self {
        Self::Jwt(jsonwebtoken::errors::new_error(
            jsonwebtoken::errors::ErrorKind::InvalidKeyFormat,
        ))
    }
}
