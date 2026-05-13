use sea_orm::DbErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Db(#[from] DbErr),

    #[error("{0}")]
    Json(#[from] serde_json::Error),

    #[error("status: {0}, msg: {1}")]
    CustomMessage(u16, String),
}

impl Error {
    pub fn with_code(code: impl Into<u16>, msg: impl Into<String>) -> Self {
        Self::CustomMessage(code.into(), msg.into())
    }
}
