use axum::{
    extract::FromRequestParts,
    http::{StatusCode, header, request::Parts},
};
use oceaniam_common::sqid::Sqid;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::{middlewares::auth::PlatformAuthGuard, state::AppState};

#[derive(Debug, Clone)]
pub struct ApplicationSecretGuard {
    pub secret: String,
    pub of_applications: Vec<Uuid>,
}

impl ApplicationSecretGuard {
    pub fn is_matched(&self, application_id: Uuid) -> bool {
        self.of_applications.contains(&application_id)
    }
}

impl FromRequestParts<AppState> for ApplicationSecretGuard {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        AppState { applications, .. }: &AppState,
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

/// NOTE: This extractor assumes the request URI contains an `applications` path segment followed by
/// a Sqid-encoded application identifier (i.e. `/…/applications/{id}/…`). Mounting this middleware
/// on a route whose path does not follow that convention will cause authorization to always fail
/// with 400 Bad Request.
#[derive(Debug, Clone)]
pub struct MatchedApplicationSecretGuard;

impl FromRequestParts<AppState> for MatchedApplicationSecretGuard {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let secret: ApplicationSecretGuard =
            ApplicationSecretGuard::from_request_parts(parts, state).await?;

        let path_segments: Vec<&str> = parts.uri.path().split('/').collect();

        let application_id = path_segments
            .iter()
            .position(|&segment| segment == "applications")
            .and_then(|idx| path_segments.get(idx + 1))
            .and_then(|id| id.parse::<Sqid>().ok())
            .and_then(|id| Uuid::try_from(id).ok())
            .ok_or_else(|| {
                warn!("application authorization failed: cannot extract application_id from path");
                StatusCode::BAD_REQUEST
            })?;

        let application_ids = state
            .applications
            .secrets()
            .find_secret_belong_to(secret.secret.clone())
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        if !application_ids.contains(&application_id) {
            warn!(
                %application_id,
                "application authorization failed: secret does not belong to application"
            );
            return Err(StatusCode::FORBIDDEN);
        }

        Ok(Self)
    }
}

/// NOTE: When the request authenticates via application secret (i.e. the
/// `X-OceanIAM-Application-Secret` header is present), this delegates to
/// `RequireMatchedApplicationSecret` and therefore inherits the same URI path constraint — the route
/// must include `/applications/{id}` in its path.
pub type AdminJwtOrApplicationSecretGuard =
    axum_extra::either::Either<PlatformAuthGuard, MatchedApplicationSecretGuard>;
