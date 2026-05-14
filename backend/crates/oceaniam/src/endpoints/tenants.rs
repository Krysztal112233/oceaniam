//! Tenant management-related API endpoints
//!
//! Provides interfaces for tenant CRUD operations

use crate::error::AppResult;
use axum::{
    Json,
    extract::{Path, State},
};
use axum_extra::extract::OptionalQuery;
use oceaniam_api::{ApiResponse, Empty, ErrorResponse, PageParam, PagedResponse};
use oceaniam_audit::types::{
    AuditPayload, CreateTenantsPayload, DeleteTenantsPayload, PatchTenantPayload,
};
use oceaniam_auth::jwt::SystemClaim;
use oceaniam_database::{
    helper::{tenants::TenantsHelper, users::UserHelper},
    model::prelude::*,
};
use oceaniam_vo::{
    applications::ApplicationUserVO,
    sqid::Sqid,
    tenants::{CreateTenantRequest, PatchTenantRequest, TenantVO},
};
use tap::Tap;
use tracing::{Span, error, field, warn};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{middlewares, state::AppState};

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState<'a>>) -> OpenApiRouter<AppState<'a>> {
    router
        .routes(routes!(create_tenant))
        .routes(routes!(delete_tenant))
        .routes(routes!(get_tenant))
        .routes(routes!(get_tenant_users))
        .routes(routes!(patch_tenant))
        .routes(routes!(get_tenants))
}

/// Get tenant list
#[utoipa::path(
        get,
        path = "/tenants",
        tag = "Tenants",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("page" = Option<u64>, Query, description = "Page number"),
            ("per_page" = Option<u64>, Query, description = "Items per page"),
        ),
        responses(
            (status = 200, body = ApiResponse<PagedResponse<TenantVO>>),
            (status = 401, description = "Unauthorized"),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenants.list",
    skip(auth, database),
    fields(operator_id = field::Empty, page = field::Empty, per_page = field::Empty)
)]
pub async fn get_tenants(
    auth: middlewares::auth::RequireAuth<SystemClaim>,
    OptionalQuery(query): OptionalQuery<PageParam>,
    State(AppState { database, .. }): State<AppState<'_>>,
) -> AppResult<PagedResponse<TenantVO>> {
    let page = query.unwrap_or_default();
    let operator_id = auth.token.claims.sub;
    Span::current().tap(|it| {
        it.record("operator_id", field::display(&operator_id))
            .record("page", page.page)
            .record("per_page", page.per_page);
    });

    let PagedResponse { items, page_info } = Tenants::get_tenants(page, &database)
        .await
        .inspect_err(|e| error!(%operator_id, error = %e, "tenant list query failed"))?;

    warn!(
        %operator_id,
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
            ("tenant_id" = String, Path, description = "Tenant ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<TenantVO>),
            (status = 401, description = "Unauthorized"),
            (status = 404, description = "Tenant not found"),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenants.get",
    skip(auth, tenant_id, database),
    fields(operator_id = field::Empty, tenant_id = field::Empty)
)]
pub async fn get_tenant(
    auth: middlewares::auth::RequireAuth<SystemClaim>,
    Path(tenant_id): Path<Sqid>,
    State(AppState { database, .. }): State<AppState<'_>>,
) -> AppResult<TenantVO> {
    let operator_id = auth.token.claims.sub;
    let uuid = tenant_id.try_into()?;
    Span::current().tap(|it| {
        it.record("operator_id", field::display(&operator_id))
            .record("tenant_id", field::display(&uuid));
    });

    let result = Tenants::get_tenant(uuid, &database)
        .await
        .inspect_err(|e| {
            error!(
                tenant_id = %uuid,
                %operator_id,
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
        request_body = CreateTenantRequest,
        responses(
            (status = 201, body = ApiResponse<TenantVO>),
            (status = 400, description = "Bad request"),
            (status = 401, description = "Unauthorized"),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenants.create",
    skip(auth, database, auditing, comment),
    fields(operator_id = field::Empty, tenant_id = field::Empty)
)]
pub async fn create_tenant(
    auth: middlewares::auth::RequireAuth<SystemClaim>,

    State(AppState {
        database, auditing, ..
    }): State<AppState<'_>>,

    Json(CreateTenantRequest { comment }): Json<CreateTenantRequest>,
) -> AppResult<TenantVO> {
    let operator_id = auth.token.claims.sub;
    let tenant_id = Uuid::now_v7();
    Span::current().tap(|it| {
        it.record("operator_id", field::display(&operator_id))
            .record("tenant_id", field::display(&tenant_id));
    });

    let model = Tenants::create_tenant(tenant_id, comment, &database)
        .await
        .inspect_err(|e| {
            warn!(
                tenant_id = %tenant_id,
                %operator_id,
                error = %e,
                "tenant creation failed"
            )
        })?;

    warn!(
        tenant_id = %tenant_id,
        %operator_id,
        "tenant created successfully"
    );

    auditing
        .write(AuditPayload::from(CreateTenantsPayload {
            tenant_id: model.id,
            comment: model.comment.clone(),
            operator_id,
        }))
        .await;

    Ok(ApiResponse::new(model.into()))
}

/// Update tenant
#[utoipa::path(
        patch,
        path = "/tenants/{tenant_id}",
        tag = "Tenants",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
        ),
        request_body = PatchTenantRequest,
        responses(
            (status = 200, body = ApiResponse<TenantVO>),
            (status = 400, description = "Bad request", body = ApiResponse<ErrorResponse>),
            (status = 401, description = "Unauthorized"),
            (status = 404, description = "Tenant not found", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenants.patch",
    skip(auth, database, auditing, tenant_id, comment),
    fields(operator_id = field::Empty, tenant_id = field::Empty)
)]
pub async fn patch_tenant(
    auth: middlewares::auth::RequireAuth<SystemClaim>,
    Path(tenant_id): Path<Sqid>,
    State(AppState {
        database, auditing, ..
    }): State<AppState<'_>>,
    Json(PatchTenantRequest { comment }): Json<PatchTenantRequest>,
) -> AppResult<TenantVO> {
    let operator_id = auth.token.claims.sub;
    let tenant_id: Uuid = tenant_id.try_into()?;
    Span::current().tap(|it| {
        it.record("operator_id", field::display(&operator_id))
            .record("tenant_id", field::display(&tenant_id));
    });

    let tenant = Tenants::update_comment(tenant_id, comment, &database)
        .await
        .inspect_err(|e| {
            error!(
                %operator_id,
                %tenant_id,
                error = %e,
                "tenant update failed"
            )
        })?;

    auditing
        .write(AuditPayload::from(PatchTenantPayload {
            tenant_id,
            operator_id,
            comment: tenant.comment.clone(),
        }))
        .await;

    Ok(ApiResponse::new(tenant.into()))
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
            ("tenant_id" = String, Path, description = "Tenant ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<Empty>),
            (status = 400, description = "Bad request"),
            (status = 401, description = "Unauthorized"),
            (status = 404, description = "Tenant not found"),
            (status = 500, description = "Internal server error"),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenants.delete",
    skip(auth, database, auditing),
    fields(operator_id = field::Empty, tenant_id = field::Empty)
)]
pub async fn delete_tenant(
    auth: middlewares::auth::RequireAuth<SystemClaim>,
    Path(tenant_id): Path<Uuid>,
    State(AppState {
        database, auditing, ..
    }): State<AppState<'_>>,
) -> AppResult<()> {
    let operator_id = auth.token.claims.sub;
    Span::current().tap(|it| {
        it.record("operator_id", field::display(&operator_id))
            .record("tenant_id", field::display(&tenant_id));
    });

    Tenants::delete_tenant(tenant_id, &database)
        .await
        .inspect_err(|e| {
            error!(
                tenant_id = %tenant_id,
                %operator_id,
                error = %e,
                "tenant deletion failed"
            )
        })?;

    warn!(
        tenant_id = %tenant_id,
        %operator_id,
        "tenant deleted successfully"
    );

    auditing
        .write(AuditPayload::from(DeleteTenantsPayload {
            tenant_id,
            operator_id,
        }))
        .await;

    Ok(ApiResponse::new(()))
}

/// Get users of tenant
#[utoipa::path(
        get,
        path = "/tenants/{tenant_id}/users",
        tag = "Tenants",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("page" = Option<u64>, Query, description = "Page number"),
            ("per_page" = Option<u64>, Query, description = "Items per page"),
        ),
        responses(
            (status = 200, body = ApiResponse<PagedResponse<ApplicationUserVO>>),
            (status = 401, description = "Unauthorized"),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_users.list",
    skip(auth, database, tenant_id),
    fields(operator_id = field::Empty, tenant_id = field::Empty, page = field::Empty, per_page = field::Empty)
)]
pub async fn get_tenant_users(
    auth: middlewares::auth::RequireAuth<SystemClaim>,
    Path(tenant_id): Path<Sqid>,
    OptionalQuery(query): OptionalQuery<PageParam>,
    State(AppState { database, .. }): State<AppState<'_>>,
) -> AppResult<PagedResponse<ApplicationUserVO>> {
    let page = query.unwrap_or_default();
    let operator_id = auth.token.claims.sub;
    let tenant_id = tenant_id.try_into()?;

    Span::current().tap(|it| {
        it.record("operator_id", field::display(&operator_id))
            .record("tenant_id", field::display(&tenant_id))
            .record("page", page.page)
            .record("per_page", page.per_page);
    });

    let PagedResponse { items, page_info } = Users::get_users_of_tenant(tenant_id, page, &database)
        .await
        .inspect_err(|e| {
            error!(
                %operator_id,
                tenant_id = %tenant_id,
                error = %e,
                "tenant user list query failed"
            )
        })?;
    let items: Vec<ApplicationUserVO> = items.into_iter().map(Into::into).collect();

    warn!(
        %operator_id,
        %tenant_id,
        count = items.len(),
        "tenant user list queried successfully"
    );

    Ok(ApiResponse::new(PagedResponse { items, page_info }))
}
