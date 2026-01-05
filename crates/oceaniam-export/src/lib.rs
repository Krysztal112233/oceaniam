pub mod database;
pub mod vo;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_common::ErrorResponse")]
pub struct ErrorResponse;
