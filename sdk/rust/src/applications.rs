use oceaniam_api::{PageParam, PagedResponse};
use oceaniam_vo::applications::*;
use reqwest::Method;
use serde::Deserialize;

use crate::client::{AuthMode, OceanIamClient};
use crate::error::Error;
use crate::paths;

/// A single JSON Web Key.
#[derive(Debug, Deserialize)]
pub struct Jwk {
    pub kty: String,
    pub kid: Option<String>,
    pub alg: Option<String>,
    #[serde(rename = "use")]
    pub use_: Option<String>,
    pub n: Option<String>,
    pub e: Option<String>,
    pub crv: Option<String>,
    pub x: Option<String>,
    pub y: Option<String>,
}

/// JWK Set — response from the JWKS endpoint.
#[derive(Debug, Deserialize)]
pub struct JwkSet {
    pub keys: Vec<Jwk>,
}

impl OceanIamClient {
    pub async fn get_applications(
        &self,
        tenant_id: &str,
        pagination: Option<&PageParam>,
    ) -> Result<PagedResponse<ApplicationVO>, Error> {
        let path = paths::fmt1(paths::TENANT_APPS, tenant_id);
        let mut req = self.auth_req(Method::GET, &path, AuthMode::Bearer)?;
        if let Some(p) = pagination {
            req = req.query(p);
        }
        self.send_inner(req).await
    }

    pub async fn create_application(
        &self,
        tenant_id: &str,
        body: &CreateApplicationRequest,
    ) -> Result<CreateApplicationResponse, Error> {
        let path = paths::fmt1(paths::TENANT_APPS, tenant_id);
        let req = self
            .auth_req(Method::POST, &path, AuthMode::Bearer)?
            .json(body);
        self.send_inner(req).await
    }

    pub async fn get_application(
        &self,
        tenant_id: &str,
        application_id: &str,
    ) -> Result<ApplicationDetailVO, Error> {
        let path = paths::fmt2(paths::TENANT_APP, tenant_id, application_id);
        let req = self.auth_req(Method::GET, &path, AuthMode::Bearer)?;
        self.send_inner(req).await
    }

    pub async fn patch_application(
        &self,
        tenant_id: &str,
        application_id: &str,
        body: &PatchApplicationRequest,
    ) -> Result<ApplicationDetailVO, Error> {
        let path = paths::fmt2(paths::TENANT_APP, tenant_id, application_id);
        let req = self
            .auth_req(Method::PATCH, &path, AuthMode::Bearer)?
            .json(body);
        self.send_inner(req).await
    }

    pub async fn delete_application(
        &self,
        tenant_id: &str,
        application_id: &str,
    ) -> Result<(), Error> {
        let path = paths::fmt2(paths::TENANT_APP, tenant_id, application_id);
        let req = self.auth_req(Method::DELETE, &path, AuthMode::Bearer)?;
        self.send_empty(req).await
    }

    pub async fn get_application_configuration(
        &self,
        tenant_id: &str,
        application_id: &str,
    ) -> Result<GetApplicationConfigurationResponse, Error> {
        let path = paths::fmt2(paths::APP_CONFIG, tenant_id, application_id);
        let req = self.auth_req(Method::GET, &path, AuthMode::Bearer)?;
        self.send_inner(req).await
    }

    pub async fn patch_application_configuration(
        &self,
        tenant_id: &str,
        application_id: &str,
        body: &PatchApplicationConfigurationRequest,
    ) -> Result<(), Error> {
        let path = paths::fmt2(paths::APP_CONFIG, tenant_id, application_id);
        let req = self
            .auth_req(Method::PATCH, &path, AuthMode::Bearer)?
            .json(body);
        self.send_empty(req).await
    }

    pub async fn get_application_jwks(&self, application_id: &str) -> Result<JwkSet, Error> {
        let path = paths::fmt1(paths::JWKS, application_id);
        let req = self.auth_req(Method::GET, &path, AuthMode::None)?;
        self.send_inner(req).await
    }
}
