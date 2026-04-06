use axum::{
    extract::FromRequestParts,
    http::{StatusCode, header, request::Parts},
};
use itertools::Either;
use oceaniam_common::{jwt::SystemClaim, types::sqid::Sqid};
use tracing::{debug, warn};
use uuid::Uuid;

use crate::{
    middlewares::{application::RequireApplicationSecret, auth::RequireAuth},
    state::AppState,
};

/// Ensures the provided application secret matches the application identifier in the path.
#[derive(Debug, Clone)]
pub struct RequireMatchedApplicationSecret;

impl FromRequestParts<AppState<'_>> for RequireMatchedApplicationSecret {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState<'_>,
    ) -> Result<Self, Self::Rejection> {
        let secret: RequireApplicationSecret =
            RequireApplicationSecret::from_request_parts(parts, state).await?;

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

/// Ensures the request is authenticated either by a system administrator JWT
/// or by an application secret that matches the application identifier in the path.
#[derive(Debug, Clone)]
pub struct RequireAdminJwtOrMatchedApplicationSecret(
    #[allow(unused)] pub Either<RequireAuth<SystemClaim>, RequireMatchedApplicationSecret>,
);

impl FromRequestParts<AppState<'_>> for RequireAdminJwtOrMatchedApplicationSecret {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState<'_>,
    ) -> Result<Self, Self::Rejection> {
        if parts.headers.contains_key(header::AUTHORIZATION) {
            return Ok(Self(Either::Left(
                RequireAuth::<SystemClaim>::from_request_parts(parts, state).await?,
            )));
        }

        if parts.headers.contains_key("X-OceanIAM-Application-Secret") {
            return Ok(Self(Either::Right(
                RequireMatchedApplicationSecret::from_request_parts(parts, state).await?,
            )));
        }

        debug!(
            "application authorization failed: missing both authorization and application secret headers"
        );
        Err(StatusCode::NON_AUTHORITATIVE_INFORMATION)
    }
}
