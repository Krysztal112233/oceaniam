use snafu::{Location, Snafu};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("{source} at {location}"))]
    Database {
        source: oceaniam_database::error::Error,
        location: Location,
    },

    #[snafu(display("{source} at {location}"))]
    DatabaseRaw {
        source: sea_orm::DbErr,
        location: Location,
    },

    #[snafu(display("{msg} at {location}"))]
    Internal { msg: String, location: Location },
}

impl From<oceaniam_database::error::Error> for Error {
    fn from(source: oceaniam_database::error::Error) -> Self {
        Error::Database {
            source,
            location: snafu::location!(),
        }
    }
}

impl From<sea_orm::DbErr> for Error {
    fn from(source: sea_orm::DbErr) -> Self {
        Error::DatabaseRaw {
            source,
            location: snafu::location!(),
        }
    }
}
