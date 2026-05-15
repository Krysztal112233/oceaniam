use oceaniam_api::PagedResponse;
use oceaniam_vo::applications::*;
use reqwest::Method;

use crate::client::{AuthMode, OceanIamClient};
use crate::error::Error;
use crate::paths;

impl OceanIamClient {
    pub async fn get_application_keys(
        &self,
        tenant_id: &str,
        application_id: &str,
    ) -> Result<PagedResponse<ApplicationKeyVO>, Error> {
        let path = paths::fmt2(paths::APP_KEYS, tenant_id, application_id);
        let req = self.auth_req(Method::GET, &path, AuthMode::BearerOrAppSecret)?;
        self.send_inner(req).await
    }

    pub async fn rotate_application_key(
        &self,
        tenant_id: &str,
        application_id: &str,
    ) -> Result<RotateKeyResponse, Error> {
        let path = paths::fmt2(paths::APP_KEYS, tenant_id, application_id);
        let req = self.auth_req(Method::POST, &path, AuthMode::Bearer)?;
        self.send_inner(req).await
    }

    pub async fn revoke_application_key(
        &self,
        tenant_id: &str,
        application_id: &str,
        key_id: &str,
    ) -> Result<(), Error> {
        let path = paths::fmt3(paths::APP_KEY, tenant_id, application_id, key_id);
        let req = self.auth_req(Method::DELETE, &path, AuthMode::Bearer)?;
        self.send_empty(req).await
    }
}
