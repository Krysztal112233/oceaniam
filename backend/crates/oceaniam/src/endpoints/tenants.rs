//! Tenant management-related API endpoints
//!
//! Provides interfaces for tenant CRUD operations

use axum::{
    Json,
    extract::{Path, Query, State},
};
use oceaniam_common::{
    ApiResponse, Empty, PagedResponse, RestResult, jwt::SystemClaim, types::sqid::Sqid,
};
use oceaniam_database::{helper::tenants::TenantsHelper, model::prelude::*};
use oceaniam_vo::tenants::{CreateTenantRequest, GetTenantsRequest, TenantVO};
use tracing::{error, warn};
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
    let span = tracing::info_span!("tenants.list", operator_id = %operator_id);
    let _guard = span.enter();

    let PagedResponse { items, page_info } =
        Tenants::get_tenants(page, &database).await.inspect_err(
            |e| error!(operator_id = %operator_id, error = %e, "tenant list query failed"),
        )?;

    warn!(
        operator_id = %operator_id,
        count = items.len(),
        "tenant list queried successfully"
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
    let span = tracing::info_span!(
        "tenants.get",
        operator_id = %operator_id,
        tenant_id = %uuid
    );
    let _guard = span.enter();

    let result = Tenants::get_tenant(uuid, &database)
        .await
        .inspect_err(|e| {
            error!(
                tenant_id = %uuid,
                operator_id = %operator_id,
                error = %e,
                "tenant query failed"
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
    let span = tracing::info_span!(
        "tenants.create",
        operator_id = %operator_id,
        tenant_id = %tenant_id
    );
    let _guard = span.enter();

    let model = Tenants::create_tenant(tenant_id, comment, &database)
        .await
        .inspect_err(|e| {
            warn!(
                tenant_id = %tenant_id,
                operator_id = %operator_id,
                error = %e,
                "tenant creation failed"
            )
        })?;

    warn!(
        tenant_id = %tenant_id,
        operator_id = %operator_id,
        "tenant created successfully"
    );

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
    let span = tracing::info_span!(
        "tenants.delete",
        operator_id = %operator_id,
        tenant_id = %tenant_id
    );
    let _guard = span.enter();

    Tenants::delete_tenant(tenant_id, &database)
        .await
        .inspect_err(|e| {
            error!(
                tenant_id = %tenant_id,
                operator_id = %operator_id,
                error = %e,
                "tenant deletion failed"
            )
        })?;

    warn!(
        tenant_id = %tenant_id,
        operator_id = %operator_id,
        "tenant deleted successfully"
    );

    Ok(ApiResponse::new(()))
}
