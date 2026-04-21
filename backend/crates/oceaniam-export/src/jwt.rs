#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_auth::jwt::Claim")]
pub struct Claim;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_auth::jwt::SystemClaim")]
pub struct SystemClaim;
