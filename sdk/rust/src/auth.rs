use oceaniam_vo::applications::ApplicationChallengeVO;
use oceaniam_vo::auth::*;
use reqwest::Method;
use serde_json::Value;

use crate::client::{AuthMode, OceanIamClient};
use crate::error::Error;
use crate::paths;

impl OceanIamClient {
    // -- system auth --
    pub async fn system_signin(
        &self,
        body: &SystemSigninRequest,
        dispatch: Option<&str>,
    ) -> Result<SigninResponseOrChallenge, Error> {
        let mut req = self
            .auth_req(Method::POST, paths::AUTH_TOKENS, AuthMode::None)?
            .json(body);
        if let Some(d) = dispatch {
            req = req.header("X-OceanIAM-Token-Dispatch", d);
        }
        self.send_inner(req).await
    }

    pub async fn system_signout(&self) -> Result<Value, Error> {
        let req = self.auth_req(Method::DELETE, paths::AUTH_TOKENS, AuthMode::Bearer)?;
        self.send_inner(req).await
    }

    pub async fn system_refresh_token(
        &self,
        dispatch: Option<&str>,
    ) -> Result<SigninResponseOrChallenge, Error> {
        let mut req = self.auth_req(Method::POST, paths::AUTH_TOKENS_REFRESH, AuthMode::Bearer)?;
        if let Some(d) = dispatch {
            req = req.header("X-OceanIAM-Token-Dispatch", d);
        }
        self.send_inner(req).await
    }

    // -- application tokens --
    pub async fn application_user_signin(
        &self,
        tenant_id: &str,
        application_id: &str,
        body: &AuthVO,
        dispatch: Option<&str>,
    ) -> Result<SigninResponseOrChallenge, Error> {
        let path = paths::fmt2(paths::APP_TOKENS, tenant_id, application_id);
        let mut req = self
            .auth_req(Method::POST, &path, AuthMode::AppSecret)?
            .json(body);
        if let Some(d) = dispatch {
            req = req.header("X-OceanIAM-Token-Dispatch", d);
        }
        self.send_inner(req).await
    }

    pub async fn application_user_signout(
        &self,
        tenant_id: &str,
        application_id: &str,
    ) -> Result<Value, Error> {
        let path = paths::fmt2(paths::APP_TOKENS, tenant_id, application_id);
        let req = self.auth_req(Method::DELETE, &path, AuthMode::AppSecret)?;
        self.send_inner(req).await
    }

    pub async fn application_user_refresh_token(
        &self,
        tenant_id: &str,
        application_id: &str,
        dispatch: Option<&str>,
    ) -> Result<SigninResponseOrChallenge, Error> {
        let path = paths::fmt2(paths::APP_TOKENS_REFRESH, tenant_id, application_id);
        let mut req = self.auth_req(Method::POST, &path, AuthMode::AppSecret)?;
        if let Some(d) = dispatch {
            req = req.header("X-OceanIAM-Token-Dispatch", d);
        }
        self.send_inner(req).await
    }

    // -- application challenges --
    pub async fn get_application_challenge(
        &self,
        tenant_id: &str,
        application_id: &str,
        challenge_id: &str,
    ) -> Result<ApplicationChallengeVO, Error> {
        let path = paths::fmt3(
            paths::APP_CHALLENGE,
            tenant_id,
            application_id,
            challenge_id,
        );
        let req = self.auth_req(Method::GET, &path, AuthMode::BearerOrAppSecret)?;
        self.send_inner(req).await
    }

    pub async fn submit_application_challenge(
        &self,
        tenant_id: &str,
        application_id: &str,
        challenge_id: &str,
        payload: &Value,
        dispatch: Option<&str>,
    ) -> Result<SigninResponseOrChallenge, Error> {
        let path = paths::fmt3(
            paths::APP_CHALLENGE,
            tenant_id,
            application_id,
            challenge_id,
        );
        let mut req = self
            .auth_req(Method::POST, &path, AuthMode::BearerOrAppSecret)?
            .json(payload);
        if let Some(d) = dispatch {
            req = req.header("X-OceanIAM-Token-Dispatch", d);
        }
        self.send_inner(req).await
    }
}
