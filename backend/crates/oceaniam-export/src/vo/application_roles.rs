#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::application_roles::ApplicationRoleVO")]
pub struct ApplicationRoleVO;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::application_roles::CreateApplicationRoleRequest")]
pub struct CreateApplicationRoleRequest;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::application_roles::PatchApplicationRoleRequest")]
pub struct PatchApplicationRoleRequest;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::application_roles::RolePermissionsVO")]
pub struct RolePermissionsVO;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::application_roles::SetRolePermissionsRequest")]
pub struct SetRolePermissionsRequest;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::application_roles::SubjectRolesVO")]
pub struct SubjectRolesVO;

#[derive(ts_rs::TS)]
#[ts(export)]
#[ts(as = "oceaniam_vo::application_roles::AssignRoleRequest")]
pub struct AssignRoleRequest;
