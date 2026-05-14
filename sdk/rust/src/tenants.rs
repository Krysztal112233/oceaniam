use oceaniam_api::{PageParam, PagedResponse};
use oceaniam_vo::applications::ApplicationUserVO;
use oceaniam_vo::tenants::*;
use reqwest::Method;

use crate::client::{AuthMode, OceanIamClient};
use crate::error::Error;
use crate::paths;

impl OceanIamClient {
    pub async fn get_tenants(
        &self,
        pagination: Option<&PageParam>,
    ) -> Result<PagedResponse<TenantVO>, Error> {
        let mut req = self.auth_req(Method::GET, paths::TENANTS, AuthMode::Bearer)?;
        if let Some(p) = pagination {
            req = req.query(p);
        }
        self.send_inner(req).await
    }

    pub async fn get_tenant(&self, tenant_id: &str) -> Result<TenantVO, Error> {
        let path = paths::fmt1(paths::TENANT, tenant_id);
        let req = self.auth_req(Method::GET, &path, AuthMode::Bearer)?;
        self.send_inner(req).await
    }

    pub async fn create_tenant(&self, body: &CreateTenantRequest) -> Result<TenantVO, Error> {
        let req = self
            .auth_req(Method::POST, paths::TENANTS, AuthMode::Bearer)?
            .json(body);
        self.send_inner(req).await
    }

    pub async fn patch_tenant(
        &self,
        tenant_id: &str,
        body: &PatchTenantRequest,
    ) -> Result<TenantVO, Error> {
        let path = paths::fmt1(paths::TENANT, tenant_id);
        let req = self
            .auth_req(Method::PATCH, &path, AuthMode::Bearer)?
            .json(body);
        self.send_inner(req).await
    }

    pub async fn delete_tenant(&self, tenant_id: &str) -> Result<(), Error> {
        let path = paths::fmt1(paths::TENANT, tenant_id);
        let req = self.auth_req(Method::DELETE, &path, AuthMode::Bearer)?;
        self.send_empty(req).await
    }

    pub async fn get_tenant_users(
        &self,
        tenant_id: &str,
        pagination: Option<&PageParam>,
    ) -> Result<PagedResponse<ApplicationUserVO>, Error> {
        let path = paths::fmt1(paths::TENANT_USERS, tenant_id);
        let mut req = self.auth_req(Method::GET, &path, AuthMode::Bearer)?;
        if let Some(p) = pagination {
            req = req.query(p);
        }
        self.send_inner(req).await
    }
}
