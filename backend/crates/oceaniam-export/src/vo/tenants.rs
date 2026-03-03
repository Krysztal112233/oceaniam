#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::tenants::CreateTenantRequest")]
pub struct CreateTenantRequest;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::tenants::TenantVO")]
pub struct TenantVO;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::tenants::GetTenantsRequest")]
pub struct GetTenantsRequest;
