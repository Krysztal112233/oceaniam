#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_common::jwt::Claim")]
pub struct Claim;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_common::jwt::SystemClaim")]
pub struct SystemClaim;
