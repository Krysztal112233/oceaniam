use oceaniam_vo::{
    oidc_clients::{CreateOidcClientRequest, OidcClientVO},
    pagination::{PageParam, PagedResponse},
};
use reqwest::Method;

use crate::{
    client::{AuthMode, OceanIamClient},
    error::Error,
    paths,
};

impl OceanIamClient {
    pub async fn get_oidc_clients(
        &self,
        tenant_id: &str,
        application_id: &str,
        pagination: Option<&PageParam>,
    ) -> Result<PagedResponse<OidcClientVO>, Error> {
        let path = paths::fmt2(paths::APP_OIDC_CLIENTS, tenant_id, application_id);
        let mut request = self.auth_req(Method::GET, &path, AuthMode::Bearer)?;
        if let Some(pagination) = pagination {
            request = request.query(pagination);
        }
        self.send_inner(request).await
    }

    pub async fn create_oidc_client(
        &self,
        tenant_id: &str,
        application_id: &str,
        body: &CreateOidcClientRequest,
    ) -> Result<OidcClientVO, Error> {
        let path = paths::fmt2(paths::APP_OIDC_CLIENTS, tenant_id, application_id);
        let request = self
            .auth_req(Method::POST, &path, AuthMode::Bearer)?
            .json(body);
        self.send_inner(request).await
    }

    pub async fn get_oidc_client(
        &self,
        tenant_id: &str,
        application_id: &str,
        client_id: &str,
    ) -> Result<OidcClientVO, Error> {
        let path = paths::fmt3(paths::APP_OIDC_CLIENT, tenant_id, application_id, client_id);
        let request = self.auth_req(Method::GET, &path, AuthMode::Bearer)?;
        self.send_inner(request).await
    }
}
