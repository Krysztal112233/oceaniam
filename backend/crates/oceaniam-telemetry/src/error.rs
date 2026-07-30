use snafu::{Location, Snafu};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("telemetry config error: {msg}"))]
    Config {
        msg: String,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("failed to build OTLP exporter: {msg}"))]
    Exporter {
        msg: String,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("invalid env filter directive: {source}"))]
    EnvFilter {
        source: tracing_subscriber::filter::ParseError,
        #[snafu(implicit)]
        location: Location,
    },
}
