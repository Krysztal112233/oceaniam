use std::sync::Arc;

use reqwest::{Method, RequestBuilder};

use crate::error::Error;

pub(crate) enum AuthMode {
    Bearer,
    AppSecret,
    BearerOrAppSecret,
    None,
}

pub struct OceanIamClient {
    inner: reqwest::Client,
    base_url: String,
    token_getter: Option<Arc<dyn Fn() -> Option<String> + Send + Sync>>,
    app_secret: Option<String>,
}

impl OceanIamClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            inner: reqwest::Client::new(),
            base_url: base_url.into(),
            token_getter: None,
            app_secret: None,
        }
    }

    pub fn with_token_getter(
        mut self,
        getter: impl Fn() -> Option<String> + Send + Sync + 'static,
    ) -> Self {
        self.token_getter = Some(Arc::new(getter));
        self
    }

    pub fn with_app_secret(mut self, secret: impl Into<String>) -> Self {
        self.app_secret = Some(secret.into());
        self
    }

    pub(crate) fn auth_req(
        &self,
        method: Method,
        path: &str,
        auth: AuthMode,
    ) -> Result<RequestBuilder, Error> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.inner.request(method, &url);

        match auth {
            AuthMode::Bearer => {
                let token = self
                    .token_getter
                    .as_ref()
                    .and_then(|f| (f)())
                    .ok_or_else(|| Error::MissingAuthToken {
                        location: snafu::location!(),
                    })?;
                req = req.bearer_auth(token);
            }
            AuthMode::AppSecret => {
                let secret = self
                    .app_secret
                    .as_ref()
                    .ok_or_else(|| Error::MissingAppSecret {
                        location: snafu::location!(),
                    })?;
                req = req.header("X-OceanIAM-Application-Secret", secret);
            }
            AuthMode::BearerOrAppSecret => {
                if let Some(token) = self.token_getter.as_ref().and_then(|f| (f)()) {
                    req = req.bearer_auth(token);
                }
                if let Some(secret) = &self.app_secret {
                    req = req.header("X-OceanIAM-Application-Secret", secret);
                }
            }
            AuthMode::None => {}
        }

        Ok(req)
    }

    pub(crate) async fn send_inner<T: serde::de::DeserializeOwned>(
        &self,
        req: RequestBuilder,
    ) -> Result<T, Error> {
        let resp = req.send().await?;
        let status = resp.status();
        let body = resp.text().await?;

        if status.is_success() {
            Ok(serde_json::from_str(&body)?)
        } else {
            let msg = serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|v| v.get("msg").and_then(|m| m.as_str().map(String::from)))
                .unwrap_or(body.clone());
            Err(Error::Api {
                status: status.as_u16(),
                message: msg,
                location: snafu::location!(),
            })
        }
    }

    pub(crate) async fn send_empty(&self, req: RequestBuilder) -> Result<(), Error> {
        self.send_inner::<serde_json::Value>(req).await?;
        Ok(())
    }
}
