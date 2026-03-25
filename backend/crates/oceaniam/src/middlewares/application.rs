use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use tracing::warn;
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct RequireApplicationSecret {
    pub secret: String,
    pub of_applications: Vec<Uuid>,
}

impl RequireApplicationSecret {
    pub fn is_matched(&self, application_id: Uuid) -> bool {
        self.of_applications.contains(&application_id)
    }
}

impl FromRequestParts<AppState<'_>> for RequireApplicationSecret {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        AppState { applications, .. }: &AppState<'_>,
    ) -> Result<Self, Self::Rejection> {
        let header_present = parts.headers.contains_key("X-OceanIAM-Application-Secret");
        let span = tracing::debug_span!("app_secret.extract", header_present);
        let _guard = span.enter();

        let secret = parts
            .headers
            .get("X-OceanIAM-Application-Secret")
            .and_then(|value| value.to_str().ok());

        let Some(secret) = secret else {
            warn!(
                header = "X-OceanIAM-Application-Secret",
                "application authentication failed: missing application secret header"
            );
            return Err(StatusCode::UNAUTHORIZED);
        };

        let Ok(application_ids) = applications
            .secrets()
            .find_secret_belong_to(secret)
            .await
            .inspect_err(|e| warn!(error = %e, "application authentication failed: invalid application secret"))
        else {
            return Err(StatusCode::UNAUTHORIZED);
        };

        Ok(Self {
            secret: secret.to_owned(),
            of_applications: application_ids,
        })
    }
}
