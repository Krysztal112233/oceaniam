//! Tenant management-related API endpoints
//!
//! Provides interfaces for tenant CRUD operations

use axum::{
    Json,
    extract::{Path, Query, State},
};
use log::{error, warn};
use oceaniam_common::{
    ApiResponse, Empty, PagedResponse, RestResult, jwt::SystemClaim, types::sqid::Sqid,
};
use oceaniam_database::{helper::tenants::TenantsHelper, model::prelude::*};
use oceaniam_vo::tenants::{CreateTenantRequest, GetTenantsRequest, TenantVO};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{middlewares, state::AppState};

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState<'a>>) -> OpenApiRouter<AppState<'a>> {
    router
        .routes(routes!(get_tenants))
        .routes(routes!(get_tenant))
        .routes(routes!(create_tenant))
        .routes(routes!(delete_tenant))
}

/// Get tenant list
#[utoipa::path(
        get,
        path = "/tenants",
        tag = "Tenants",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
        ),
        responses(
            (status = 200, body = ApiResponse<PagedResponse<TenantVO>>),
            (status = 401, description = "Unauthorized"),
        ),
    )]
pub async fn get_tenants(
    auth: middlewares::auth::RequireAuth<SystemClaim>,

    Query(page): Query<GetTenantsRequest>,

    State(AppState { database, .. }): State<AppState<'_>>,
) -> RestResult<PagedResponse<TenantVO>> {
    let operator_id = auth.token.claims.sub;

    let PagedResponse { items, page_info } = Tenants::get_tenants(page, &database)
        .await
        .inspect_err(|e| {
            error!("tenant list query failed: operator_id={operator_id}, error={e}",)
        })?;

    warn!(
        "tenant list queried successfully: operator_id={operator_id}, count={}",
        items.len()
    );

    Ok(ApiResponse::new(PagedResponse {
        items: items.into_iter().map(Into::into).collect(),
        page_info,
    }))
}

/// Get tenant by ID
///
/// Retrieves detailed information of a specific tenant
#[utoipa::path(
        get,
        path = "/tenants/{tenant_id}",
        tag = "Tenants",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("tenant_id", description = "Tenant ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<TenantVO>),
            (status = 401, description = "Unauthorized"),
            (status = 404, description = "Tenant not found"),
        ),
    )]
pub async fn get_tenant(
    auth: middlewares::auth::RequireAuth<SystemClaim>,
    Path(tenant_id): Path<Sqid>,
    State(AppState { database, .. }): State<AppState<'_>>,
) -> RestResult<TenantVO> {
    let operator_id = auth.token.claims.sub;
    let uuid = tenant_id.try_into()?;

    let result = Tenants::get_tenant(uuid, &database)
        .await
        .inspect_err(|e| {
            error!(
                "tenant query failed: tenant_id={}, operator_id={}, error={}",
                uuid, operator_id, e
            )
        })?;

    Ok(ApiResponse::new(TenantVO::from(result)))
}

/// Create a new tenant
///
/// Creates a tenant with the specified ID
#[utoipa::path(
        post,
        path = "/tenants",
        tag = "Tenants",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
        ),
        responses(
            (status = 201, body = ApiResponse<TenantVO>),
            (status = 400, description = "Bad request"),
            (status = 401, description = "Unauthorized"),
        ),
    )]
pub async fn create_tenant(
    auth: middlewares::auth::RequireAuth<SystemClaim>,

    State(AppState { database, .. }): State<AppState<'_>>,

    Json(CreateTenantRequest { comment }): Json<CreateTenantRequest>,
) -> RestResult<TenantVO> {
    let operator_id = auth.token.claims.sub;
    let tenant_id = Uuid::now_v7();

    let model = Tenants::create_tenant(tenant_id, comment, &database)
        .await
        .inspect_err(|e| {
            warn!("tenant creation failed: tenant_id={tenant_id}, operator_id={operator_id}, error={e}")
        })?;

    warn!("tenant created successfully: tenant_id={tenant_id}, operator_id={operator_id}",);

    Ok(ApiResponse::new(model.into()))
}

/// Delete a tenant
///
/// Deletes a tenant by its ID
#[utoipa::path(
        delete,
        path = "/tenants/{tenant_id}",
        tag = "Tenants",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("tenant_id", description = "Tenant ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<Empty>),
        ),
    )]
pub async fn delete_tenant(
    auth: middlewares::auth::RequireAuth<SystemClaim>,
    Path(tenant_id): Path<Uuid>,
    State(AppState { database, .. }): State<AppState<'_>>,
) -> RestResult<()> {
    let operator_id = auth.token.claims.sub;

    Tenants::delete_tenant(tenant_id, &database)
        .await
        .inspect_err(|e| {
            error!(
                "tenant deletion failed: tenant_id={}, operator_id={}, error={}",
                tenant_id, operator_id, e
            )
        })?;

    warn!(
        "tenant deleted successfully: tenant_id={}, operator_id={}",
        tenant_id, operator_id
    );

    Ok(ApiResponse::new(()))
}
