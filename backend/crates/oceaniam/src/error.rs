use std::sync::Arc;

use axum::{Json, http::StatusCode, response::IntoResponse};
use sea_orm::DbErr;
use serde::Serialize;
use thiserror::Error;

use oceaniam_api::RestResult;

#[derive(Debug, Serialize)]
struct ErrorResponseBody {
    msg: String,
}

#[derive(Debug, Serialize)]
struct ApiErrorResponse {
    #[serde(flatten)]
    payload: Option<ErrorResponseBody>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Conf(#[from] config::ConfigError),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Json(#[from] serde_json::Error),

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

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Self::CustomMessage(code, _) => {
                StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (
            status,
            Json(ApiErrorResponse {
                payload: Some(ErrorResponseBody {
                    msg: status.to_string(),
                }),
            }),
        )
            .into_response()
    }
}

impl From<Arc<Error>> for Error {
    fn from(value: Arc<Error>) -> Self {
        match Arc::try_unwrap(value) {
            Ok(error) => error,
            Err(arc) => match arc.as_ref() {
                Error::CustomMessage(code, msg) => Error::CustomMessage(*code, msg.clone()),
                Error::Internal(msg) => Error::Internal(msg.clone()),
                _ => Error::Internal(format!("{}", arc)),
            },
        }
    }
}

impl From<oceaniam_auth::AuthError> for Error {
    fn from(e: oceaniam_auth::AuthError) -> Self {
        match e {
            oceaniam_auth::AuthError::KidNotFound(kid) => {
                Self::with_code(400u16, format!("cannot find jwk for kid `{kid}`"))
            }
            oceaniam_auth::AuthError::Jwt(e) => Self::Internal(e.to_string()),
            oceaniam_auth::AuthError::Internal(msg) => Self::Internal(msg),
        }
    }
}

impl From<oceaniam_database::error::Error> for Error {
    fn from(e: oceaniam_database::error::Error) -> Self {
        match e {
            oceaniam_database::error::Error::Db(DbErr::RecordNotFound(record)) => {
                Self::with_code(404u16, format!("cannot find {record} in database"))
            }
            oceaniam_database::error::Error::Db(e) => Self::Internal(e.to_string()),
            oceaniam_database::error::Error::Json(e) => Self::Json(e),
            oceaniam_database::error::Error::CustomMessage(code, msg) => {
                Self::CustomMessage(code, msg)
            }
        }
    }
}

impl From<oceaniam_vo::error::Error> for Error {
    fn from(e: oceaniam_vo::error::Error) -> Self {
        match e {
            oceaniam_vo::error::Error::InvalidSqid => {
                Self::with_code(400u16, "cannot parse input id")
            }
        }
    }
}

impl From<oceaniam_keybox::error::Error> for Error {
    fn from(e: oceaniam_keybox::error::Error) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<oceaniam_credential::error::Error> for Error {
    fn from(e: oceaniam_credential::error::Error) -> Self {
        match e {
            oceaniam_credential::error::Error::Db(db_err) => Self::from(db_err),
            oceaniam_credential::error::Error::Password(_) => {
                Self::with_code(500u16, oceaniam_common::consts::USER_LOGIN_FAILED_MSG)
            }
            oceaniam_credential::error::Error::Join(_) => {
                Self::with_code(500u16, "task execution failed")
            }
            oceaniam_credential::error::Error::InvalidLength
            | oceaniam_credential::error::Error::Base64(_)
            | oceaniam_credential::error::Error::SerdeJson(_)
            | oceaniam_credential::error::Error::Aead
            | oceaniam_credential::error::Error::InvalidAlgorithm => {
                Self::with_code(500u16, "invalid totp secret data")
            }
            oceaniam_credential::error::Error::Totp(_)
            | oceaniam_credential::error::Error::SystemTime(_) => {
                Self::with_code(500u16, "invalid totp configuration")
            }
        }
    }
}

impl From<axum::http::header::InvalidHeaderValue> for Error {
    fn from(e: axum::http::header::InvalidHeaderValue) -> Self {
        Self::with_code(500u16, e.to_string())
    }
}

impl From<DbErr> for Error {
    fn from(value: DbErr) -> Self {
        match value {
            DbErr::RecordNotFound(e) => {
                Self::with_code(404u16, format!("cannot find {e} in database"))
            }
            _ => Self::Internal(value.to_string()),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        Self::Internal(e.to_string())
    }
}

impl From<oceaniam_common::error::Error> for Error {
    fn from(e: oceaniam_common::error::Error) -> Self {
        match e {
            oceaniam_common::error::Error::Conf(e) => Self::Conf(e),
            oceaniam_common::error::Error::Io(e) => Self::Io(e),
            oceaniam_common::error::Error::Json(e) => Self::Json(e),
            oceaniam_common::error::Error::Internal(msg) => Self::Internal(msg),
        }
    }
}

pub type AppResult<T> = RestResult<T, Error>;
