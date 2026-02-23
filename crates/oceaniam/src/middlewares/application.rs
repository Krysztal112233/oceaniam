use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use log::warn;
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct RequireApplicationSecret {
    pub secret: String,
    pub of_application: Uuid,
}

impl RequireApplicationSecret {
    pub fn is_matched(&self, application_id: Uuid) -> bool {
        self.of_application == application_id
    }
}

impl FromRequestParts<AppState> for RequireApplicationSecret {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        AppState {
            application_secrets,
            ..
        }: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let secret = parts
            .headers
            .get("X-OceanIAM-Application-Secret")
            .and_then(|value| value.to_str().ok());

        let Some(secret) = secret else {
            warn!(
                "application authentication failed: missing `X-OceanIAM-Application-Secret` header"
            );
            return Err(StatusCode::UNAUTHORIZED);
        };

        let Ok(application_id) = application_secrets
            .find_secret_belong(secret)
            .await
            .inspect_err(|e| {
                warn!("application authentication failed: invalid secret provided: {e}")
            })
        else {
            return Err(StatusCode::UNAUTHORIZED);
        };

        Ok(Self {
            secret: secret.to_owned(),
            of_application: application_id,
        })
    }
}
