pub mod jwt;
pub mod vo;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_api::ErrorResponse")]
pub struct ErrorResponse;
