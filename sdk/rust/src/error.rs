use snafu::{Location, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("HTTP {status}: {message} at {location}"))]
    Api {
        status: u16,
        message: String,
        location: Location,
    },

    #[snafu(display("request failed: {source} at {location}"))]
    Request {
        source: reqwest::Error,
        location: Location,
    },

    #[snafu(display("missing auth token at {location}"))]
    MissingAuthToken { location: Location },

    #[snafu(display("missing application secret at {location}"))]
    MissingAppSecret { location: Location },

    #[snafu(display("JSON deserialize error: {source} at {location}"))]
    Json {
        source: serde_json::Error,
        location: Location,
    },

    #[snafu(display("UUID conversion error: {source} at {location}"))]
    Uuid {
        source: uuid::Error,
        location: Location,
    },
}

impl From<reqwest::Error> for Error {
    fn from(source: reqwest::Error) -> Self {
        Self::Request {
            source,
            location: snafu::location!(),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(source: serde_json::Error) -> Self {
        Self::Json {
            source,
            location: snafu::location!(),
        }
    }
}

impl From<uuid::Error> for Error {
    fn from(source: uuid::Error) -> Self {
        Self::Uuid {
            source,
            location: snafu::location!(),
        }
    }
}
