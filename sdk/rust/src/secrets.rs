use oceaniam_vo::pagination::{PageParam, PagedResponse};
use oceaniam_vo::applications::SecretVO;
use reqwest::Method;

use crate::client::{AuthMode, OceanIamClient};
use crate::error::Error;
use crate::paths;

impl OceanIamClient {
    pub async fn create_secret(&self) -> Result<SecretVO, Error> {
        let req = self.auth_req(Method::POST, paths::SECRETS, AuthMode::Bearer)?;
        self.send_inner(req).await
    }

    pub async fn get_secrets(
        &self,
        pagination: Option<&PageParam>,
    ) -> Result<PagedResponse<SecretVO>, Error> {
        let mut req = self.auth_req(Method::GET, paths::SECRETS, AuthMode::Bearer)?;
        if let Some(p) = pagination {
            req = req.query(p);
        }
        self.send_inner(req).await
    }

    pub async fn get_secret(&self, secret_id: &str) -> Result<SecretVO, Error> {
        let path = paths::fmt1(paths::SECRET, secret_id);
        let req = self.auth_req(Method::GET, &path, AuthMode::Bearer)?;
        self.send_inner(req).await
    }

    pub async fn delete_secret(&self, secret_id: &str) -> Result<(), Error> {
        let path = paths::fmt1(paths::SECRET, secret_id);
        let req = self.auth_req(Method::DELETE, &path, AuthMode::Bearer)?;
        self.send_empty(req).await
    }
}
