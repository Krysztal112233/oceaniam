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

impl From<config::ConfigError> for Error {
    fn from(source: config::ConfigError) -> Self {
        Error::Conf {
            source,
            location: snafu::location!(),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(source: std::io::Error) -> Self {
        Error::Io {
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
