use snafu::{Location, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{source} at {location}"))]
    Jwt {
        source: jsonwebtoken::errors::Error,
        location: Location,
    },

    #[snafu(display("kid not found: {kid} at {location}"))]
    KidNotFound { kid: String, location: Location },

    #[snafu(display("{msg} at {location}"))]
    Internal { msg: String, location: Location },
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(source: jsonwebtoken::errors::Error) -> Self {
        Error::Jwt {
            source,
            location: snafu::location!(),
        }
    }
}
