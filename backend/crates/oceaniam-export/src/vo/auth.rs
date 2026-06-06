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
#[ts(as = "oceaniam_vo::auth::SignupResponse")]
pub struct SigninResponse;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::auth::SigninResponseOrChallenge")]
pub struct SigninResponseSchema;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::auth::SignoutResponse")]
pub struct SignoutResponse;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::auth::SystemSigninRequest")]
pub struct SystemSigninRequest;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::auth::SigninResponseOrChallenge")]
pub struct SystemSigninResponse;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::auth::SigninChallenge")]
pub struct SigninChallenge;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::auth::EnrollTotpResponse")]
pub struct EnrollTotpResponse;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::auth::VerifyTotpRequest")]
pub struct VerifyTotpRequest;
