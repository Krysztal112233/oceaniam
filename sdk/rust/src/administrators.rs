use oceaniam_vo::administrators::*;
use oceaniam_vo::pagination::{PageParam, PagedResponse};
use reqwest::Method;

use crate::client::{AuthMode, OceanIamClient};
use crate::error::Error;
use crate::paths;

impl OceanIamClient {
    pub async fn get_administrators(
        &self,
        pagination: Option<&PageParam>,
    ) -> Result<PagedResponse<AdministratorVO>, Error> {
        let mut req = self.auth_req(Method::GET, paths::ADMINISTRATORS, AuthMode::Bearer)?;
        if let Some(p) = pagination {
            req = req.query(p);
        }
        self.send_inner(req).await
    }

    pub async fn create_administrator(
        &self,
        body: &CreateAdministratorRequest,
    ) -> Result<CreateAdministratorResponse, Error> {
        let req = self
            .auth_req(Method::POST, paths::ADMINISTRATORS, AuthMode::Bearer)?
            .json(body);
        self.send_inner(req).await
    }

    pub async fn patch_administrator(
        &self,
        administrator_id: &str,
        body: &PatchAdministratorRequest,
    ) -> Result<AdministratorVO, Error> {
        let path = paths::fmt1(paths::ADMINISTRATOR, administrator_id);
        let req = self
            .auth_req(Method::PATCH, &path, AuthMode::Bearer)?
            .json(body);
        self.send_inner(req).await
    }

    pub async fn get_administrator_self(&self) -> Result<AdministratorProfileVO, Error> {
        let req = self.auth_req(Method::GET, paths::ADMINISTRATOR_SELF, AuthMode::Bearer)?;
        self.send_inner(req).await
    }
}
