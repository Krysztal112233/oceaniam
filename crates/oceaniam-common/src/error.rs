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
    Unknown(String),

    #[error("status: {0}, msg: {1}")]
    UnknownWithCode(u16, String),
}

#[allow(clippy::match_single_binding)]
impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (
            status,
            ApiResponse::new(ErrorResponse::new(status.to_string())),
        )
            .into_response()
    }
}
