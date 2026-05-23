use oceaniam_vo::pagination::PagedResponse;
use oceaniam_vo::applications::*;
use reqwest::Method;

use crate::client::{AuthMode, OceanIamClient};
use crate::error::Error;
use crate::paths;

impl OceanIamClient {
    pub async fn get_application_users(
        &self,
        tenant_id: &str,
        application_id: &str,
        query: Option<&ApplicationUsersListQuery>,
    ) -> Result<PagedResponse<ApplicationUserVO>, Error> {
        let path = paths::fmt2(paths::APP_USERS, tenant_id, application_id);
        let mut req = self.auth_req(Method::GET, &path, AuthMode::BearerOrAppSecret)?;
        if let Some(q) = query {
            req = req.query(q);
        }
        self.send_inner(req).await
    }

    pub async fn search_application_users(
        &self,
        tenant_id: &str,
        application_id: &str,
        query: &SearchApplicationUsersQuery,
    ) -> Result<PagedResponse<ApplicationUserVO>, Error> {
        let path = paths::fmt2(paths::APP_USERS_SEARCH, tenant_id, application_id);
        let req = self
            .auth_req(Method::GET, &path, AuthMode::BearerOrAppSecret)?
            .query(query);
        self.send_inner(req).await
    }

    pub async fn get_application_user(
        &self,
        tenant_id: &str,
        application_id: &str,
        user_id: &str,
    ) -> Result<ApplicationUserVO, Error> {
        let path = paths::fmt3(paths::APP_USER, tenant_id, application_id, user_id);
        let req = self.auth_req(Method::GET, &path, AuthMode::BearerOrAppSecret)?;
        self.send_inner(req).await
    }

    pub async fn create_application_user(
        &self,
        tenant_id: &str,
        application_id: &str,
        body: &CreateApplicationUserRequest,
    ) -> Result<ApplicationUserVO, Error> {
        let path = paths::fmt2(paths::APP_USERS, tenant_id, application_id);
        let req = self
            .auth_req(Method::POST, &path, AuthMode::BearerOrAppSecret)?
            .json(body);
        self.send_inner(req).await
    }

    pub async fn patch_application_user_credentials(
        &self,
        tenant_id: &str,
        application_id: &str,
        user_id: &str,
        body: &PatchApplicationUserCredentialsRequest,
    ) -> Result<ApplicationUserVO, Error> {
        let path = paths::fmt3(paths::APP_USER_CREDS, tenant_id, application_id, user_id);
        let req = self
            .auth_req(Method::PATCH, &path, AuthMode::BearerOrAppSecret)?
            .json(body);
        self.send_inner(req).await
    }
}
