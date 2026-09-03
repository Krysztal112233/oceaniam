//! OIDC protocol endpoints.
//!
//! Version one exposes only the tenant JWKS under the OIDC namespace. The Discovery document,
//! Authorization/Token/UserInfo endpoints, and issuer metadata are deliberately out of scope
//! until their full slices are implemented.

use crate::error::AppResult;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, header},
};
use oceaniam_api::{ApiResponse, ErrorResponse};
use oceaniam_auth::jwks::JwkSetSchema;
use oceaniam_auth::oidc::{CoreJsonWebKeySet, core_jwk_set};
use oceaniam_common::sqid::Sqid;
use oceaniam_database::helper::tenants::TenantsHelper;
use tap::{Tap, TapFallible};
use tracing::{Span, error, field};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::state::AppState;

/// Public HTTP cache lifetime for tenant JWK Sets. A conservative bound that keeps key rotation
/// visible to clients without relying on cache invalidation.
const JWKS_CACHE_CONTROL: &str = "public, max-age=60";

/// Tenant JWK Set under the OIDC namespace.
///
/// The tenant is resolved through the system-protected lookup, so the system tenant never
/// appears in the OIDC namespace, and the response body is the official
/// [`CoreJsonWebKeySet`] type from `openidconnect`.
#[utoipa::path(
    get,
    path = "/oidc/{tenant_sqid}/jwks.json",
    tag = "Oidc",
    params(
        ("tenant_sqid" = String, Path, description = "Tenant Sqid"),
    ),
    responses(
        (
            status = 200,
            body = JwkSetSchema,
            headers(
                ("Cache-Control" = String, description = "Bounded public cache lifetime for the JWK Set"),
            ),
        ),
        (status = 400, description = "Invalid tenant id", body = ApiResponse<ErrorResponse>),
        (status = 404, description = "Tenant not found", body = ApiResponse<ErrorResponse>),
        (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
    ),
)]
#[tracing::instrument(
    level = "info",
    name = "oidc.jwks",
    skip_all,
    fields(otel.kind = "internal", tenant_id = field::Empty)
)]
pub async fn get_oidc_jwks(
    Path(tenant_sqid): Path<Sqid>,
    State(AppState {
        database, keyboxes, ..
    }): State<AppState>,
) -> AppResult<CoreJsonWebKeySet> {
    let tenant_id: Uuid = tenant_sqid
        .try_into()
        .tap_err(|e| error!(error = %e, "failed to convert tenant sqid"))?;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&tenant_id));
    });

    // System-protected lookup: rejects the system tenant and unknown tenants with 404 before
    // any keybox access, so no keys are generated for absent tenants.
    <oceaniam_database::model::prelude::Tenants as TenantsHelper>::get_tenant(tenant_id, &database)
        .await
        .inspect_err(|e| error!(%tenant_id, error = %e, "failed to resolve tenant"))?;

    let jwks = keyboxes
        .get_jwks(tenant_id)
        .await
        .inspect_err(|e| error!(%tenant_id, error = %e, "failed to get jwks"))?;
    let core_jwks = core_jwk_set(&jwks)
        .inspect_err(|e| error!(%tenant_id, error = %e, "failed to convert jwks"))?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(JWKS_CACHE_CONTROL),
    );

    Ok(ApiResponse::with_header(core_jwks, headers))
}

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router.routes(routes!(get_oidc_jwks))
}
