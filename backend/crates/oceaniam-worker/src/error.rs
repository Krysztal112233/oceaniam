use snafu::{Location, Snafu};

use sea_orm::DbErr;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{msg} at {location}"))]
    Internal { msg: String, location: Location },

    #[snafu(display("{source} at {location}"))]
    Db { source: DbErr, location: Location },
}

impl From<DbErr> for Error {
    fn from(source: DbErr) -> Self {
        Error::Db {
            source,
            location: snafu::location!(),
        }
    }
}

impl From<oceaniam_common::error::Error> for Error {
    fn from(e: oceaniam_common::error::Error) -> Self {
        Error::Internal {
            msg: e.to_string(),
            location: snafu::location!(),
        }
    }
}

impl From<oceaniam_database::error::Error> for Error {
    fn from(e: oceaniam_database::error::Error) -> Self {
        Error::Internal {
            msg: e.to_string(),
            location: snafu::location!(),
        }
    }
}
