#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::administrators::AdministratorVO")]
pub struct AdministratorVO;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::administrators::CreateAdministratorRequest")]
pub struct CreateAdministratorRequest;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::administrators::CreateAdministratorResponse")]
pub struct CreateAdministratorResponse;
