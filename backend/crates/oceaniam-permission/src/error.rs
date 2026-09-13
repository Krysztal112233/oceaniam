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

oceaniam_common::located_from!(oceaniam_database::error::Error => Error::Database);

oceaniam_common::located_from!(sea_orm::DbErr => Error::DatabaseRaw);
