use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("invalid sqid"))]
    InvalidSqid,
}
