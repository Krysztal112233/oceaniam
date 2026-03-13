//! Tenant management-related API endpoints
//!
//! Provides interfaces for tenant CRUD operations

use axum::{
    Json,
    extract::{Path, State},
};
use oceaniam_audit::types::{AuditPayload, CreateTenantsPayload, DeleteTenantsPayload};
use oceaniam_common::{
    ApiResponse, Empty, PagedResponse, RestResult, jwt::SystemClaim, types::sqid::Sqid,
};
use oceaniam_database::{helper::tenants::TenantsHelper, model::prelude::*};
use oceaniam_vo::tenants::{CreateTenantRequest, TenantVO};
use tap::Tap;
use tracing::{Span, error, field, warn};
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
#[tracing::instrument(
    level = "info",
    name = "tenants.list",
    skip(auth, database),
    fields(operator_id = field::Empty)
)]
pub async fn get_tenants(
    auth: middlewares::auth::RequireAuth<SystemClaim>,

    State(AppState { database, .. }): State<AppState<'_>>,
) -> RestResult<PagedResponse<TenantVO>> {
    let operator_id = auth.token.claims.sub;
    Span::current().tap(|it| {
        it.record("operator_id", field::display(&operator_id));
    });

    let PagedResponse { items, page_info } =
        Tenants::get_all_tenants(&database).await.inspect_err(
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
) -> RestResult<TenantVO> {
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
) -> RestResult<TenantVO> {
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

    auditing
        .write(AuditPayload::from(CreateTenantsPayload {
            tenant_id: model.id,
            comment: model.comment.clone(),
            operator_id: Some(operator_id),
        }))
        .await;

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
) -> RestResult<()> {
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

    auditing
        .write(AuditPayload::from(DeleteTenantsPayload {
            tenant_id,
            operator_id: Some(operator_id),
        }))
        .await;

    Ok(ApiResponse::new(()))
}
