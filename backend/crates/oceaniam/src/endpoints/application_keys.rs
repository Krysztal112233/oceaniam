//! Application key management API endpoints

use crate::error::AppResult;
use axum::extract::{Path, State};
use oceaniam_api::{ApiResponse, Empty, ErrorResponse, PagedResponse};
use oceaniam_audit::types::{AuditPayload, RevokeKeyPayload, RotateKeyPayload};
use oceaniam_vo::applications::{ApplicationKeyVO, RotateKeyResponse};
use oceaniam_vo::sqid::Sqid;
use tap::Tap;
use tracing::{Span, error, field, info};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    middlewares::application::AdminJwtOrApplicationSecretGuard,
    middlewares::auth::RequireAuthGuard, state::AppState,
};
use oceaniam_auth::jwt::SystemClaim;

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub(crate) struct KeysPath {
    tenant_id: Sqid,
    application_id: Sqid,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub(crate) struct KeyPath {
    tenant_id: Sqid,
    application_id: Sqid,
    key_id: Sqid,
}

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState<'a>>) -> OpenApiRouter<AppState<'a>> {
    router
        .routes(routes!(get_application_keys))
        .routes(routes!(rotate_application_key))
        .routes(routes!(revoke_application_key))
}

/// List all keys for an application
#[utoipa::path(
        get,
        path = "/tenants/{tenant_id}/applications/{application_id}/keys",
        tag = "ApplicationKeys",
        params(
            ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
            ("X-OceanIAM-Application-Secret" = String, Header, description = "Application secret"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("application_id" = String, Path, description = "Application ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<PagedResponse<ApplicationKeyVO>>),
            (status = 203, description = "Missing Authorization header and application secret header"),
            (status = 400, description = "Invalid ids", body = ApiResponse<ErrorResponse>),
            (status = 401, description = "Unauthorized"),
            (status = 403, description = "Forbidden - secret does not belong to this application"),
            (status = 404, description = "Application not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_keys.list",
    skip(keyboxes, path),
    fields(tenant_id = field::Empty, application_id = field::Empty)
)]
pub async fn get_application_keys(
    _: AdminJwtOrApplicationSecretGuard,

    Path(path): Path<KeysPath>,
    State(AppState { keyboxes, .. }): State<AppState<'_>>,
) -> AppResult<PagedResponse<ApplicationKeyVO>> {
    let application_id: Uuid = path.application_id.try_into().inspect_err(|e| {
        error!(error = %e, "failed to convert application_id");
    })?;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&path.tenant_id))
            .record("application_id", field::display(&application_id));
    });

    let keybox = keyboxes
        .get_keybox(application_id)
        .await
        .inspect_err(|e| error!(%application_id, error = %e, "failed to get keybox"))?;

    let keys = keybox
        .get_keys()
        .values()
        .cloned()
        .map(ApplicationKeyVO::from)
        .collect::<Vec<_>>();

    info!(
        application_id = %application_id,
        key_count = keys.len(),
        "listed application keys"
    );

    Ok(ApiResponse::new(PagedResponse::with_entire(keys)))
}

/// Rotate (generate a new) key for an application
#[utoipa::path(
        post,
        path = "/tenants/{tenant_id}/applications/{application_id}/keys",
        tag = "ApplicationKeys",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("application_id" = String, Path, description = "Application ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<RotateKeyResponse>),
            (status = 400, description = "Invalid ids", body = ApiResponse<ErrorResponse>),
            (status = 404, description = "Application not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_keys.rotate",
    skip(keyboxes, auditing, path),
    fields(tenant_id = field::Empty, application_id = field::Empty, key_id = field::Empty)
)]
pub async fn rotate_application_key(
    _: RequireAuthGuard<SystemClaim>,

    Path(path): Path<KeysPath>,
    State(AppState {
        keyboxes, auditing, ..
    }): State<AppState<'_>>,
) -> AppResult<RotateKeyResponse> {
    let application_id: Uuid = path.application_id.try_into().inspect_err(|e| {
        error!(error = %e, "failed to convert application_id");
    })?;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&path.tenant_id))
            .record("application_id", field::display(&application_id));
    });

    let new_key = keyboxes
        .rotate_key(application_id)
        .await
        .inspect_err(|e| error!(%application_id, error = %e, "key rotation failed"))?;

    Span::current().tap(|it| {
        it.record("key_id", field::display(&new_key.id));
    });

    info!(
        %application_id,
        key_id = %new_key.id,
        "key rotated successfully"
    );

    auditing
        .write(AuditPayload::from(RotateKeyPayload {
            application_id,
            new_key_id: new_key.id,
        }))
        .await;

    Ok(ApiResponse::new(RotateKeyResponse {
        key: ApplicationKeyVO::from(new_key),
    }))
}

/// Revoke a specific key for an application
#[utoipa::path(
        delete,
        path = "/tenants/{tenant_id}/applications/{application_id}/keys/{key_id}",
        tag = "ApplicationKeys",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("application_id" = String, Path, description = "Application ID"),
            ("key_id" = String, Path, description = "Key ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<Empty>),
            (status = 400, description = "Invalid ids", body = ApiResponse<ErrorResponse>),
            (status = 404, description = "Key or application not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_keys.revoke",
    skip(keyboxes, auditing, path),
    fields(tenant_id = field::Empty, application_id = field::Empty, key_id = field::Empty)
)]
pub async fn revoke_application_key(
    _: RequireAuthGuard<SystemClaim>,
    Path(path): Path<KeyPath>,
    State(AppState {
        keyboxes, auditing, ..
    }): State<AppState<'_>>,
) -> AppResult<()> {
    let application_id: Uuid = path.application_id.try_into().inspect_err(|e| {
        error!(error = %e, "failed to convert application_id");
    })?;

    let key_id: Uuid = path.key_id.try_into().inspect_err(|e| {
        error!(error = %e, "failed to convert key_id");
    })?;

    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&path.tenant_id))
            .record("application_id", field::display(&application_id))
            .record("key_id", field::display(&key_id));
    });

    keyboxes
        .revoke_key(application_id, key_id)
        .await
        .inspect_err(|e| error!(%application_id, %key_id, error = %e, "key revocation failed"))?;

    info!(
        %application_id,
        %key_id,
        "key revoked successfully"
    );

    auditing
        .write(AuditPayload::from(RevokeKeyPayload {
            application_id,
            key_id,
        }))
        .await;

    Ok(ApiResponse::new(()))
}
