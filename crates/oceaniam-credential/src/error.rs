use argon2::password_hash;
use axum::http::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Db(#[from] sea_orm::error::DbErr),

    #[error("{0}")]
    Password(#[from] password_hash::Error),
}

impl From<Error> for oceaniam_common::error::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::Db(db_err) => Self::Db(db_err),
            Error::Password(_) => Self::with_code(
                StatusCode::INTERNAL_SERVER_ERROR,
                oceaniam_common::consts::USER_LOGIN_FAILED_MSG,
            ),
        }
    }
}
