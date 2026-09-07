use oceaniam_vo::{
    oidc_clients::{CreateOidcClientRequest, OidcClientVO, PatchOidcClientRequest},
    pagination::{PageParam, PagedResponse},
};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use reqwest::Method;

use crate::{
    client::{AuthMode, OceanIamClient},
    error::Error,
    paths,
};

const PATH_SEGMENT_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

fn oidc_client_path(tenant_id: &str, application_id: &str, client_id: &str) -> String {
    let client_id = utf8_percent_encode(client_id, PATH_SEGMENT_ENCODE_SET).to_string();
    paths::fmt3(
        paths::APP_OIDC_CLIENT,
        tenant_id,
        application_id,
        &client_id,
    )
}

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
        let path = oidc_client_path(tenant_id, application_id, client_id);
        let request = self.auth_req(Method::GET, &path, AuthMode::Bearer)?;
        self.send_inner(request).await
    }

    pub async fn patch_oidc_client(
        &self,
        tenant_id: &str,
        application_id: &str,
        client_id: &str,
        body: &PatchOidcClientRequest,
    ) -> Result<OidcClientVO, Error> {
        let path = oidc_client_path(tenant_id, application_id, client_id);
        let request = self
            .auth_req(Method::PATCH, &path, AuthMode::Bearer)?
            .json(body);
        self.send_inner(request).await
    }

    pub async fn delete_oidc_client(
        &self,
        tenant_id: &str,
        application_id: &str,
        client_id: &str,
    ) -> Result<(), Error> {
        let path = oidc_client_path(tenant_id, application_id, client_id);
        let request = self.auth_req(Method::DELETE, &path, AuthMode::Bearer)?;
        self.send_empty(request).await
    }
}

#[cfg(test)]
mod tests {
    use oceaniam_vo::{oidc_clients::PatchOidcClientRequest, patch::PatchValue};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    use super::{OceanIamClient, oidc_client_path};
    use crate::error::Error;

    // NOTE: AI-generated test
    #[test]
    fn oidc_client_patch_model_and_item_path_keep_client_ids_opaque() {
        let patch = PatchOidcClientRequest {
            name: PatchValue::Missing,
            redirect_uris: PatchValue::Value(vec![
                "https://CLIENT.example/callback/%2Fraw".to_owned(),
            ]),
        };

        assert_eq!(
            serde_json::to_value(patch).unwrap(),
            serde_json::json!({
                "redirect_uris": ["https://CLIENT.example/callback/%2Fraw"]
            })
        );
        assert_eq!(
            oidc_client_path("tenant", "application", "client?revision=1/part#%2F"),
            "/tenants/tenant/applications/application/oidc-clients/client%3Frevision%3D1%2Fpart%23%252F"
        );
    }

    // NOTE: AI-generated test
    #[tokio::test]
    async fn delete_oidc_client_accepts_an_empty_204_response() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = vec![0; 4096];
            let read = stream.read(&mut request).await.unwrap();
            stream
                .write_all(b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n")
                .await
                .unwrap();
            String::from_utf8(request[..read].to_vec()).unwrap()
        });
        let client = OceanIamClient::new(format!("http://{address}"))
            .with_token_getter(|| Some("test-token".to_owned()));

        client
            .delete_oidc_client("tenant", "application", "opaque.client-id")
            .await
            .unwrap();

        let request = server.await.unwrap();
        assert!(request.starts_with(
            "DELETE /tenants/tenant/applications/application/oidc-clients/opaque.client-id HTTP/1.1"
        ));
        assert!(
            request
                .to_ascii_lowercase()
                .contains("authorization: bearer test-token")
        );
    }

    // NOTE: AI-generated test
    #[tokio::test]
    async fn patch_oidc_client_maps_203_to_an_api_error_and_encodes_the_item_segment() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = vec![0; 4096];
            let read = stream.read(&mut request).await.unwrap();
            let body = r#"{"msg":"missing authorization"}"#;
            let response = format!(
                "HTTP/1.1 203 Non-Authoritative Information\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).await.unwrap();
            String::from_utf8(request[..read].to_vec()).unwrap()
        });
        let client = OceanIamClient::new(format!("http://{address}"))
            .with_token_getter(|| Some("test-token".to_owned()));

        let error = client
            .patch_oidc_client(
                "tenant",
                "application",
                "client?revision=1/part#%2F",
                &PatchOidcClientRequest::default(),
            )
            .await
            .expect_err("203 must be treated as an API error");
        assert!(matches!(
            error,
            Error::Api {
                status: 203,
                ref message,
                ..
            } if message == "missing authorization"
        ));

        let request = server.await.unwrap();
        assert!(request.starts_with(
            "PATCH /tenants/tenant/applications/application/oidc-clients/client%3Frevision%3D1%2Fpart%23%252F HTTP/1.1"
        ));
    }
}
