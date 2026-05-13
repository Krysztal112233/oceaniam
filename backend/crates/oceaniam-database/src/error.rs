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
}

impl From<DbErr> for Error {
    fn from(source: DbErr) -> Self {
        Error::Db {
            source,
            location: snafu::location!(),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(source: serde_json::Error) -> Self {
        Error::Json {
            source,
            location: snafu::location!(),
        }
    }
}
