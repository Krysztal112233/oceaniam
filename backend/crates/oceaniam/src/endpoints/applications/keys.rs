use axum::extract::{Path, State};
use axum::http::StatusCode;
use oceaniam_api::{ApiResponse, Empty, ErrorResponse, PagedResponse};
use oceaniam_audit::types::{AuditPayload, RevokeKeyPayload, RotateKeyPayload};
use oceaniam_common::sqid::Sqid;
use oceaniam_database::model::sea_orm_active_enums::KeyStatus;
use oceaniam_vo::applications::ApplicationKeyVO;
use tap::Tap;
use tracing::{Span, error, field, info};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    error::{AppResult, Error},
    middlewares::permission::{KeyRead, KeyRevoke, KeyRotate, PlatformPermissionGuard},
    state::AppState,
};

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub(crate) struct KeysPath {
    tenant_id: Sqid,
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub(crate) struct KeyPath {
    tenant_id: Sqid,
    key_id: Sqid,
}

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router
        .routes(routes!(get_tenant_keys))
        .routes(routes!(rotate_tenant_key))
        .routes(routes!(revoke_tenant_key))
}

/// List all keys for a tenant
#[utoipa::path(
        get,
        path = "/tenants/{tenant_id}/keys",
        tag = "TenantKeys",
        params(
            ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<PagedResponse<ApplicationKeyVO>>),
            (status = 203, description = "Missing Authorization header"),
            (status = 400, description = "Invalid ids", body = ApiResponse<ErrorResponse>),
            (status = 401, description = "Unauthorized"),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_keys.list",
    skip(keyboxes, path),
    fields(otel.kind = "internal", tenant_id = field::Empty)
)]
pub async fn get_tenant_keys(
    _: PlatformPermissionGuard<KeyRead>,
    Path(path): Path<KeysPath>,
    State(AppState { keyboxes, .. }): State<AppState>,
) -> AppResult<PagedResponse<ApplicationKeyVO>> {
    let tenant_id: Uuid = path.tenant_id.try_into().inspect_err(|e| {
        error!(error = %e, "failed to convert tenant_id");
    })?;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&tenant_id));
    });

    let keybox = keyboxes
        .get_keybox(tenant_id)
        .await
        .inspect_err(|e| error!(%tenant_id, error = %e, "failed to get keybox"))?;

    let keys = keybox
        .get_keys()
        .values()
        .cloned()
        .map(crate::conversion::keys::key_model_to_vo)
        .collect::<Vec<_>>();

    info!(
        %tenant_id,
        key_count = keys.len(),
        "listed tenant keys"
    );

    Ok(ApiResponse::new(PagedResponse::with_entire(keys)))
}

/// Rotate (generate a new) key for a tenant
#[utoipa::path(
        post,
        path = "/tenants/{tenant_id}/keys",
        tag = "TenantKeys",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<Empty>),
            (status = 400, description = "Invalid ids", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_keys.rotate",
    skip(keyboxes, auditing, path),
    fields(otel.kind = "internal", tenant_id = field::Empty)
)]
pub async fn rotate_tenant_key(
    _: PlatformPermissionGuard<KeyRotate>,
    Path(path): Path<KeysPath>,
    State(AppState {
        keyboxes, auditing, ..
    }): State<AppState>,
) -> AppResult<()> {
    let tenant_id: Uuid = path.tenant_id.try_into().inspect_err(|e| {
        error!(error = %e, "failed to convert tenant_id");
    })?;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&tenant_id));
    });

    keyboxes
        .rotate_key(tenant_id)
        .await
        .inspect_err(|e| error!(%tenant_id, error = %e, "key rotation failed"))?;

    let key_id = {
        let keybox = keyboxes.get_keybox(tenant_id).await?;
        keybox
            .get_latest_raw_key(KeyStatus::Active)
            .ok_or_else(|| {
                Error::with_code(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "no active key after rotation",
                )
            })?
            .key_id
    };

    info!(
        %tenant_id,
        %key_id,
        "key rotated successfully"
    );

    auditing
        .write(AuditPayload::from(RotateKeyPayload {
            application_id: tenant_id,
            new_key_id: key_id,
        }))
        .await;

    Ok(ApiResponse::new(()))
}

/// Revoke a specific key for a tenant
#[utoipa::path(
        delete,
        path = "/tenants/{tenant_id}/keys/{key_id}",
        tag = "TenantKeys",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("key_id" = String, Path, description = "Key ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<Empty>),
            (status = 400, description = "Invalid ids", body = ApiResponse<ErrorResponse>),
            (status = 404, description = "Key not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_keys.revoke",
    skip(keyboxes, auditing, path),
    fields(otel.kind = "internal", tenant_id = field::Empty, key_id = field::Empty)
)]
pub async fn revoke_tenant_key(
    _: PlatformPermissionGuard<KeyRevoke>,
    Path(path): Path<KeyPath>,
    State(AppState {
        keyboxes, auditing, ..
    }): State<AppState>,
) -> AppResult<()> {
    let tenant_id: Uuid = path.tenant_id.try_into().inspect_err(|e| {
        error!(error = %e, "failed to convert tenant_id");
    })?;

    let key_id: Uuid = path.key_id.try_into().inspect_err(|e| {
        error!(error = %e, "failed to convert key_id");
    })?;

    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&tenant_id))
            .record("key_id", field::display(&key_id));
    });

    keyboxes
        .revoke_key(tenant_id, key_id)
        .await
        .inspect_err(|e| error!(%tenant_id, %key_id, error = %e, "key revocation failed"))?;

    info!(
        %tenant_id,
        %key_id,
        "key revoked successfully"
    );

    auditing
        .write(AuditPayload::from(RevokeKeyPayload {
            application_id: tenant_id,
            key_id,
        }))
        .await;

    Ok(ApiResponse::new(()))
}
