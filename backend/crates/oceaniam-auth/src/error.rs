use snafu::{Location, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{source} at {location}"))]
    Jwt {
        source: oceaniam_crypto::JwtError,
        location: Location,
    },

    #[snafu(display("kid not found: {kid} at {location}"))]
    KidNotFound { kid: String, location: Location },

    #[snafu(display("{msg} at {location}"))]
    Internal { msg: String, location: Location },
}

impl From<oceaniam_crypto::JwtError> for Error {
    fn from(source: oceaniam_crypto::JwtError) -> Self {
        Error::Jwt {
            source,
            location: snafu::location!(),
        }
    }
}
