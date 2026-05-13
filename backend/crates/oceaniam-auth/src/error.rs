use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{source}"), context(false))]
    Jwt { source: jsonwebtoken::errors::Error },

    #[snafu(display("kid not found: {kid}"))]
    KidNotFound { kid: String },

    #[snafu(display("{msg}"))]
    Internal { msg: String },
}
