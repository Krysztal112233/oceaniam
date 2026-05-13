use sea_orm::DbErr;
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{source}"), context(false))]
    Db { source: DbErr },

    #[snafu(display("{source}"), context(false))]
    Json { source: serde_json::Error },

    #[snafu(display("status: {code}, msg: {msg}"))]
    CustomMessage { code: u16, msg: String },
}

impl Error {
    pub fn with_code(code: impl Into<u16>, msg: impl Into<String>) -> Self {
        Self::CustomMessage {
            code: code.into(),
            msg: msg.into(),
        }
    }
}
