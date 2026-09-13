use sea_orm::DbErr;
use snafu::{Location, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{source} at {location}"))]
    Db { source: DbErr, location: Location },

    #[snafu(display("{source} at {location}"))]
    Json {
        source: serde_json::Error,
        location: Location,
    },

    #[snafu(display("status: {code}, msg: {msg} at {location}"))]
    CustomMessage {
        code: u16,
        msg: String,
        location: Location,
    },
}

oceaniam_common::located_with_code!(Error::CustomMessage);

impl Error {
    #[track_caller]
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::with_code(axum::http::StatusCode::NOT_FOUND, msg)
    }
}

oceaniam_common::located_from!(DbErr => Error::Db);

oceaniam_common::located_from!(serde_json::Error => Error::Json);

#[cfg(test)]
#[path = "error_location_tests.rs"]
mod tests;
