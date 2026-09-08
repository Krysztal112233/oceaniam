use oceaniam_vo::applications::*;
use oceaniam_vo::auth::{EnrollTotpResponse, VerifyTotpRequest};
use oceaniam_vo::pagination::PagedResponse;
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

    /// Creates an application user.
    ///
    /// Omitting `body.development` creates a permanent user. Supplying it creates a time-limited
    /// development account; `DevAccountOptions::default()` uses the default 3600-second TTL.
    pub async fn create_application_user(
        &self,
        tenant_id: &str,
        application_id: &str,
        body: &CreateApplicationUserRequest,
    ) -> Result<CreatedApplicationUserVO, Error> {
        let path = paths::fmt2(paths::APP_USERS, tenant_id, application_id);
        let req = self
            .auth_req(Method::POST, &path, AuthMode::BearerOrAppSecret)?
            .json(body);
        self.send_inner(req).await
    }

    pub async fn enroll_totp(
        &self,
        tenant_id: &str,
        application_id: &str,
        user_id: &str,
    ) -> Result<EnrollTotpResponse, Error> {
        let path = paths::fmt3(
            paths::APP_USER_TOTP_ENROLL,
            tenant_id,
            application_id,
            user_id,
        );
        let req = self.auth_req(Method::POST, &path, AuthMode::BearerOrAppSecret)?;
        self.send_inner(req).await
    }

    pub async fn verify_totp_enrollment(
        &self,
        tenant_id: &str,
        application_id: &str,
        user_id: &str,
        code: &str,
    ) -> Result<(), Error> {
        let path = paths::fmt3(
            paths::APP_USER_TOTP_VERIFY,
            tenant_id,
            application_id,
            user_id,
        );
        let body = VerifyTotpRequest {
            code: code.to_string(),
        };
        let req = self
            .auth_req(Method::POST, &path, AuthMode::BearerOrAppSecret)?
            .json(&body);
        self.send_empty(req).await
    }

    pub async fn remove_totp(
        &self,
        tenant_id: &str,
        application_id: &str,
        user_id: &str,
    ) -> Result<(), Error> {
        let path = paths::fmt3(paths::APP_USER_TOTP, tenant_id, application_id, user_id);
        let req = self.auth_req(Method::DELETE, &path, AuthMode::BearerOrAppSecret)?;
        self.send_empty(req).await
    }

    pub async fn patch_application_user(
        &self,
        tenant_id: &str,
        application_id: &str,
        user_id: &str,
        body: &PatchApplicationUserRequest,
    ) -> Result<ApplicationUserVO, Error> {
        let path = paths::fmt3(paths::APP_USER, tenant_id, application_id, user_id);
        let req = self
            .auth_req(Method::PATCH, &path, AuthMode::BearerOrAppSecret)?
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

    pub async fn delete_application_user(
        &self,
        tenant_id: &str,
        application_id: &str,
        user_id: &str,
    ) -> Result<(), Error> {
        let path = paths::fmt3(paths::APP_USER, tenant_id, application_id, user_id);
        let req = self.auth_req(Method::DELETE, &path, AuthMode::BearerOrAppSecret)?;
        self.send_empty(req).await
    }
}

#[cfg(test)]
mod tests {
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    use super::OceanIamClient;
    use crate::error::Error;

    async fn serve_user_response(
        body: &'static str,
    ) -> (OceanIamClient, tokio::task::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = vec![0; 4096];
            let read = stream.read(&mut request).await.unwrap();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).await.unwrap();
            String::from_utf8(request[..read].to_vec()).unwrap()
        });
        let client = OceanIamClient::new(format!("http://{address}"))
            .with_token_getter(|| Some("test-token".to_owned()));
        (client, server)
    }

    // NOTE: AI-generated test
    #[tokio::test]
    async fn get_application_user_preserves_resource_id_and_oidc_sub() {
        let (client, server) = serve_user_response(
            r#"{"id":"user-sqid","oidc_sub":"018f3f47-7b2f-7000-8000-000000000001","email":"user@example.com","phone":null,"nickname":"user"}"#,
        )
        .await;

        let user = client
            .get_application_user("tenant", "application", "user-sqid")
            .await
            .unwrap();
        assert_eq!(user.id, "user-sqid");
        assert_eq!(user.oidc_sub, "018f3f47-7b2f-7000-8000-000000000001");
        let request = server.await.unwrap();
        assert!(
            request.starts_with(
                "GET /tenants/tenant/applications/application/users/user-sqid HTTP/1.1"
            )
        );
    }

    // NOTE: AI-generated test
    #[tokio::test]
    async fn get_application_user_rejects_response_without_oidc_sub() {
        let (client, server) = serve_user_response(
            r#"{"id":"user-sqid","email":"user@example.com","phone":null,"nickname":"user"}"#,
        )
        .await;

        let error = client
            .get_application_user("tenant", "application", "user-sqid")
            .await
            .expect_err("oidc_sub is a required response field");
        assert!(matches!(error, Error::Json { .. }));
        server.await.unwrap();
    }
}
