use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use oceaniam_common::types::sqid::Sqid;
use tracing::warn;
use uuid::Uuid;

use crate::{middlewares::application::RequireApplicationSecret, state::AppState};

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
