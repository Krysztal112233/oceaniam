use std::sync::Arc;

use axum::{Json, http::StatusCode, response::IntoResponse};
use sea_orm::DbErr;
use serde::Serialize;
use snafu::{Location, Snafu};

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

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{source}"), context(false))]
    Conf { source: config::ConfigError },

    #[snafu(display("{source}"), context(false))]
    Io { source: std::io::Error },

    #[snafu(display("{source}"), context(false))]
    Json { source: serde_json::Error },

    #[snafu(display("{msg}"))]
    Internal { msg: String, location: Location },

    #[snafu(display("status: {code}, msg: {msg}"))]
    CustomMessage {
        code: u16,
        msg: String,
        location: Location,
    },
}

impl Error {
    #[track_caller]
    pub fn with_code(code: impl Into<u16>, msg: impl Into<String>) -> Self {
        let loc = std::panic::Location::caller();
        Self::CustomMessage {
            code: code.into(),
            msg: msg.into(),
            location: Location::new(loc.file(), loc.line(), loc.column()),
        }
    }

    #[track_caller]
    fn capture_location() -> Location {
        let loc = std::panic::Location::caller();
        Location::new(loc.file(), loc.line(), loc.column())
    }
}

impl Clone for Error {
    fn clone(&self) -> Self {
        match self {
            Error::Conf { .. } | Error::Io { .. } | Error::Json { .. } => Error::Internal {
                msg: self.to_string(),
                location: Location::new("", 0, 0),
            },
            Error::Internal { msg, location } => Error::Internal {
                msg: msg.clone(),
                location: *location,
            },
            Error::CustomMessage {
                code,
                msg,
                location,
            } => Error::CustomMessage {
                code: *code,
                msg: msg.clone(),
                location: *location,
            },
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Self::CustomMessage { code, .. } => {
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
    fn from(arc: Arc<Error>) -> Self {
        Arc::unwrap_or_clone(arc)
    }
}

impl From<oceaniam_auth::AuthError> for Error {
    #[track_caller]
    fn from(e: oceaniam_auth::AuthError) -> Self {
        match e {
            oceaniam_auth::AuthError::KidNotFound { kid } => {
                Self::with_code(400u16, format!("cannot find jwk for kid `{kid}`"))
            }
            oceaniam_auth::AuthError::Jwt { source } => Self::Internal {
                msg: source.to_string(),
                location: Self::capture_location(),
            },
            oceaniam_auth::AuthError::Internal { msg } => Self::Internal {
                msg,
                location: Self::capture_location(),
            },
        }
    }
}

impl From<oceaniam_database::error::Error> for Error {
    #[track_caller]
    fn from(e: oceaniam_database::error::Error) -> Self {
        match e {
            oceaniam_database::error::Error::Db {
                source: DbErr::RecordNotFound(record),
            } => Self::with_code(404u16, format!("cannot find {record} in database")),
            oceaniam_database::error::Error::Db { source: e } => Self::Internal {
                msg: e.to_string(),
                location: Self::capture_location(),
            },
            oceaniam_database::error::Error::Json { source: e } => Self::Json { source: e },
            oceaniam_database::error::Error::CustomMessage { code, msg } => Self::CustomMessage {
                code,
                msg,
                location: Self::capture_location(),
            },
        }
    }
}

impl From<oceaniam_vo::error::Error> for Error {
    #[track_caller]
    fn from(e: oceaniam_vo::error::Error) -> Self {
        match e {
            oceaniam_vo::error::Error::InvalidSqid => {
                Self::with_code(400u16, "cannot parse input id")
            }
        }
    }
}

impl From<oceaniam_keybox::error::Error> for Error {
    #[track_caller]
    fn from(e: oceaniam_keybox::error::Error) -> Self {
        Self::Internal {
            msg: e.to_string(),
            location: Self::capture_location(),
        }
    }
}

impl From<oceaniam_credential::error::Error> for Error {
    #[track_caller]
    fn from(e: oceaniam_credential::error::Error) -> Self {
        match e {
            oceaniam_credential::error::Error::Db { source: db_err } => Self::from(db_err),
            oceaniam_credential::error::Error::Password { .. } => {
                Self::with_code(500u16, oceaniam_common::consts::USER_LOGIN_FAILED_MSG)
            }
            oceaniam_credential::error::Error::Join { .. } => {
                Self::with_code(500u16, "task execution failed")
            }
            oceaniam_credential::error::Error::InvalidLength
            | oceaniam_credential::error::Error::Base64 { .. }
            | oceaniam_credential::error::Error::SerdeJson { .. }
            | oceaniam_credential::error::Error::Aead
            | oceaniam_credential::error::Error::InvalidAlgorithm => {
                Self::with_code(500u16, "invalid totp secret data")
            }
            oceaniam_credential::error::Error::Totp { .. }
            | oceaniam_credential::error::Error::SystemTime { .. } => {
                Self::with_code(500u16, "invalid totp configuration")
            }
        }
    }
}

impl From<axum::http::header::InvalidHeaderValue> for Error {
    #[track_caller]
    fn from(e: axum::http::header::InvalidHeaderValue) -> Self {
        Self::CustomMessage {
            code: 500,
            msg: e.to_string(),
            location: Self::capture_location(),
        }
    }
}

impl From<DbErr> for Error {
    #[track_caller]
    fn from(value: DbErr) -> Self {
        match value {
            DbErr::RecordNotFound(e) => {
                Self::with_code(404u16, format!("cannot find {e} in database"))
            }
            _ => Self::Internal {
                msg: value.to_string(),
                location: Self::capture_location(),
            },
        }
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    #[track_caller]
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        Self::Internal {
            msg: e.to_string(),
            location: Self::capture_location(),
        }
    }
}

impl From<oceaniam_common::error::Error> for Error {
    #[track_caller]
    fn from(e: oceaniam_common::error::Error) -> Self {
        match e {
            oceaniam_common::error::Error::Conf { source: e } => Self::Conf { source: e },
            oceaniam_common::error::Error::Io { source: e } => Self::Io { source: e },
            oceaniam_common::error::Error::Json { source: e } => Self::Json { source: e },
            oceaniam_common::error::Error::Internal { msg } => Self::Internal {
                msg,
                location: Self::capture_location(),
            },
        }
    }
}

pub type AppResult<T> = RestResult<T, Error>;
