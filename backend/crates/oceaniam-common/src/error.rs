use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{source}"), context(false))]
    Conf { source: config::ConfigError },

    #[snafu(display("{source}"), context(false))]
    Io { source: std::io::Error },

    #[snafu(display("{source}"), context(false))]
    Json { source: serde_json::Error },

    #[snafu(display("{msg}"))]
    Internal { msg: String },
}
