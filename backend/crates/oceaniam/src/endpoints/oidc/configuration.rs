//! OpenID Provider Configuration endpoints (OIDC Discovery 1.0 §4, RFC 8414 §3).
//!
//! The tenant-scoped discovery document is served on both well-known path shapes with an
//! identical body: the append style (`/oidc/{tenant_sqid}/.well-known/openid-configuration`,
//! OIDC Discovery 1.0 §4.1) and the insert style
//! (`/.well-known/openid-configuration/oidc/{tenant_sqid}`, RFC 8414 §3.1), because the two
//! specifications conflict and deployed clients implement either. Like the tenant JWKS, the
//! document is public metadata fetched cross-origin, so it lives inside the API CORS layer
//! with a bounded public cache lifetime. Discovery is gated by the same default-disabled
//! authorization-entry preview flag as the protocol family it describes: with the preview
//! disabled, both well-known paths answer 404 rather than advertise endpoints that would 404.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, header},
};
use oceaniam_api::{ApiResponse, ErrorResponse};
use oceaniam_common::sqid::Sqid;
use oceaniam_database::{helper::tenants::TenantsHelper, model::prelude::Tenants};
use oceaniam_oidc::OpenIdProviderConfiguration;
use tap::{Tap, TapFallible};
use tracing::{Span, error, field};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{error::AppResult, state::AppState};

use super::{PUBLIC_METADATA_CACHE_CONTROL, authorize::PreviewEnabled, authorize::tenant_issuer};

/// OpenID Provider Configuration on the append-style well-known path (OIDC Discovery 1.0
/// §4.1). Gated by the authorization-entry preview flag, like the protocol family it
/// describes.
///
/// The tenant is resolved through the system-protected lookup, so the system tenant never
/// appears in the OIDC namespace, and the response body is the exact mirror of the
/// implemented provider capability.
#[utoipa::path(
    get,
    path = "/oidc/{tenant_sqid}/.well-known/openid-configuration",
    tag = "Oidc",
    params(
        ("tenant_sqid" = String, Path, description = "Tenant Sqid"),
    ),
    responses(
        (
            status = 200,
            body = OpenIdProviderConfiguration,
            headers(
                ("Cache-Control" = String, description = "Bounded public cache lifetime for the provider configuration"),
            ),
        ),
        (status = 400, description = "Invalid tenant id", body = ApiResponse<ErrorResponse>),
        (status = 404, description = "Preview disabled or tenant not found", body = ApiResponse<ErrorResponse>),
        (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
    ),
)]
#[tracing::instrument(
    level = "info",
    name = "oidc.configuration",
    skip_all,
    fields(otel.kind = "internal", tenant_id = field::Empty)
)]
async fn get_oidc_configuration(
    _enabled: PreviewEnabled,
    Path(tenant_sqid): Path<Sqid>,
    State(state): State<AppState>,
) -> AppResult<OpenIdProviderConfiguration> {
    serve_configuration(tenant_sqid, state).await
}

/// OpenID Provider Configuration on the insert-style well-known path (RFC 8414 §3.1).
///
/// Same document and semantics as the append-style path; both shapes exist because the OIDC
/// Discovery and RFC 8414 path composition rules conflict for issuer URLs with a path
/// component.
#[utoipa::path(
    get,
    path = "/.well-known/openid-configuration/oidc/{tenant_sqid}",
    tag = "Oidc",
    params(
        ("tenant_sqid" = String, Path, description = "Tenant Sqid"),
    ),
    responses(
        (
            status = 200,
            body = OpenIdProviderConfiguration,
            headers(
                ("Cache-Control" = String, description = "Bounded public cache lifetime for the provider configuration"),
            ),
        ),
        (status = 400, description = "Invalid tenant id", body = ApiResponse<ErrorResponse>),
        (status = 404, description = "Preview disabled or tenant not found", body = ApiResponse<ErrorResponse>),
        (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
    ),
)]
#[tracing::instrument(
    level = "info",
    name = "oidc.configuration",
    skip_all,
    fields(otel.kind = "internal", tenant_id = field::Empty)
)]
async fn get_well_known_oidc_configuration(
    _enabled: PreviewEnabled,
    Path(tenant_sqid): Path<Sqid>,
    State(state): State<AppState>,
) -> AppResult<OpenIdProviderConfiguration> {
    serve_configuration(tenant_sqid, state).await
}

/// utoipa-axum registers one axum route per handler, so the two well-known path shapes are
/// two route entries over the single shared handler below.
pub(super) fn endpoint<'a: 'static>(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router
        .routes(routes!(get_oidc_configuration))
        .routes(routes!(get_well_known_oidc_configuration))
}

async fn serve_configuration(
    tenant_sqid: Sqid,
    AppState {
        database,
        public_base_url,
        ..
    }: AppState,
) -> AppResult<OpenIdProviderConfiguration> {
    let tenant_id: Uuid = tenant_sqid
        .try_into()
        .tap_err(|e| error!(error = %e, "failed to convert tenant sqid"))?;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&tenant_id));
    });

    // System-protected lookup: rejects the system tenant and unknown tenants with 404 before
    // the issuer is derived, so no provider metadata is served for absent tenants.
    <Tenants as TenantsHelper>::get_tenant(tenant_id, &database)
        .await
        .inspect_err(|e| error!(%tenant_id, error = %e, "failed to resolve tenant"))?;

    let issuer = tenant_issuer(&public_base_url, &Sqid::from(tenant_id));
    let configuration = OpenIdProviderConfiguration::new(issuer);

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(PUBLIC_METADATA_CACHE_CONTROL),
    );

    Ok(ApiResponse::with_header(configuration, headers))
}
