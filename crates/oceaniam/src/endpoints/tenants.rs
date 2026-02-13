//! Tenant management-related API endpoints
//!
//! Provides interfaces for tenant CRUD operations

use axum::extract::Path;
use oceaniam_common::{ApiResponse, Empty, RestResult};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::state::AppState;

pub fn endpoint(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
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
        responses(
            (status = 200, body = ApiResponse<Empty>),
        ),
    )]

pub async fn get_tenants() -> RestResult<()> {
    Ok(ApiResponse::new(()))
}

/// Get tenant by ID
///
/// Retrieves detailed information of a specific tenant
#[utoipa::path(
        get,
        path = "/tenants/{tenant_id}",
        tag = "Tenants",
        params(
            ("tenant_id", description = "Tenant ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<Empty>),
        ),
    )]

pub async fn get_tenant(Path(tenant_id): Path<Uuid>) -> RestResult<()> {
    Ok(ApiResponse::new(()))
}

/// Create a new tenant
///
/// Creates a tenant with the specified ID
#[utoipa::path(
        post,
        path = "/tenants/{tenant_id}",
        tag = "Tenants",
        params(
            ("tenant_id", description = "Tenant ID"),
        ),
        responses(
            (status = 201, body = ApiResponse<Empty>),
        ),
    )]
pub async fn create_tenant(Path(tenant_id): Path<Uuid>) -> RestResult<()> {
    Ok(ApiResponse::new(()))
}

/// Delete a tenant
///
/// Deletes a tenant by its ID
#[utoipa::path(
        delete,
        path = "/tenants/{tenant_id}",
        tag = "Tenants",
        params(
            ("tenant_id", description = "Tenant ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<Empty>),
        ),
    )]
pub async fn delete_tenant(Path(tenant_id): Path<Uuid>) -> RestResult<()> {
    Ok(ApiResponse::new(()))
}
