pub mod auth;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::auth::AuthVO")]
pub struct AuthVO;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::auth::SignupResponse")]
pub struct SignupResponse;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::auth::SigninRequest")]
pub struct SigninRequest;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::auth::SigninResponse")]
pub struct SigninResponse;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::auth::SignoutResponse")]
pub struct SignoutResponse;
