#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::applications::CreateApplicationRequest")]
pub struct CreateApplicationRequest;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::applications::CreateApplicationResponse")]
pub struct CreateApplicationResponse;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::applications::ApplicationVO")]
pub struct ApplicationVO;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::applications::ApplicationDetailVO")]
pub struct ApplicationDetailVO;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::applications::GetApplicationConfigurationResponse")]
pub struct GetApplicationConfigurationResponse;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::applications::CreateApplicationUserRequest")]
pub struct CreateApplicationUserRequest;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::applications::ApplicationUserVO")]
pub struct ApplicationUserVO;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::applications::SecretVO")]
pub struct SecretVO;
