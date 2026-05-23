#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::tenants::CreateTenantRequest")]
pub struct CreateTenantRequest;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(type = "{ comment?: string | null }")]
pub struct PatchTenantRequest;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::tenants::TenantVO")]
pub struct TenantVO;
