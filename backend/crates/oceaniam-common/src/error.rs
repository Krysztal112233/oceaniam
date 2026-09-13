use snafu::{Location, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{source} at {location}"))]
    Conf {
        source: config::ConfigError,
        location: Location,
    },

    #[snafu(display("{source} at {location}"))]
    Io {
        source: std::io::Error,
        location: Location,
    },

    #[snafu(display("{source} at {location}"))]
    Json {
        source: serde_json::Error,
        location: Location,
    },

    #[snafu(display("{msg} at {location}"))]
    Internal { msg: String, location: Location },
}

crate::located_from!(config::ConfigError => Error::Conf);

crate::located_from!(std::io::Error => Error::Io);

crate::located_from!(serde_json::Error => Error::Json);
