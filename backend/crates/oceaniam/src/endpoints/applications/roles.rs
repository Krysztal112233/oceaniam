use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use oceaniam_api::ApiResponse;
use oceaniam_common::sqid::Sqid;
use oceaniam_database::{
    helper::{role_permissions::RolePermissionsHelper, subject_roles::SubjectRolesHelper},
    model::{self, prelude::*},
};
use oceaniam_vo::{
    application_roles::{
        ApplicationRoleVO, AssignRoleRequest, CreateApplicationRoleRequest,
        PatchApplicationRoleRequest, RolePermissionsVO, SetRolePermissionsRequest, SubjectRolesVO,
    },
    pagination::PagedResponse,
};
use sea_orm::{ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use tap::Tap;
use tracing::{Span, error, field, info};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::ResolvedApplication;
use crate::{
    conversion::application_roles::application_role_model_to_vo,
    error::{AppResult, Error},
    middlewares::app_permission::{AppPermissionGuard, AppUserRead},
    state::AppState,
};

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState<'a>>) -> OpenApiRouter<AppState<'a>> {
    router
        .routes(routes!(list_roles))
        .routes(routes!(create_role))
        .routes(routes!(get_role))
        .routes(routes!(patch_role))
        .routes(routes!(delete_role))
        .routes(routes!(get_role_permissions))
        .routes(routes!(set_role_permissions))
        .routes(routes!(get_subject_roles))
        .routes(routes!(assign_role))
        .routes(routes!(unassign_role))
}

fn ensure_belongs_to_app(
    model: &model::application_roles::Model,
    app_id: Uuid,
) -> Result<(), Error> {
    if model.application_id != app_id {
        return Err(Error::with_code(
            StatusCode::NOT_FOUND,
            "role not found in this application",
        ));
    }
    Ok(())
}

async fn ensure_subject_belongs_to_app(
    subject_id: Uuid,
    application_id: Uuid,
    database: &impl oceaniam_database::helper::SafeTransactionConnectionTrait,
) -> Result<(), Error> {
    let subject = Subjects::find_by_id(subject_id)
        .one(database)
        .await
        .inspect_err(|e| {
            error!(%subject_id, error = %e, "failed to find subject");
        })?
        .ok_or_else(|| Error::with_code(StatusCode::NOT_FOUND, "subject not found"))?;

    if subject.application_id != application_id {
        return Err(Error::with_code(
            StatusCode::FORBIDDEN,
            "subject does not belong to this application",
        ));
    }
    Ok(())
}

/// List all roles for the current application
#[utoipa::path(
    get,
    path = "/tenants/{tenant_id}/applications/{application_id}/roles",
    tag = "ApplicationRoles",
    params(
        ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
        ("tenant_id" = String, Path, description = "Tenant ID"),
        ("application_id" = String, Path, description = "Application ID"),
    ),
    responses(
        (status = 200, body = ApiResponse<PagedResponse<ApplicationRoleVO>>),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error"),
    ),
)]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_roles.list",
    skip(_auth, database),
    fields(operator_id = field::Empty, tenant_id = field::Empty, application_id = field::Empty)
)]
pub async fn list_roles(
    _auth: AppPermissionGuard<AppUserRead>,
    app: ResolvedApplication,
    State(AppState { database, .. }): State<AppState<'_>>,
) -> AppResult<PagedResponse<ApplicationRoleVO>> {
    let application_id = app.id();
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id));
    });

    let roles = ApplicationRoles::find()
        .filter(model::application_roles::Column::ApplicationId.eq(application_id))
        .all(&database)
        .await
        .inspect_err(|e| {
            error!(%application_id, error = %e, "failed to list application roles");
        })?;

    let mut vos = Vec::with_capacity(roles.len());
    for role in roles {
        let permissions = RolePermissions::get_role_permissions(role.id, &database)
            .await
            .inspect_err(|e| {
                error!(role_id = %role.id, error = %e, "failed to get role permissions");
            })?;
        vos.push(application_role_model_to_vo(role, permissions));
    }

    Ok(ApiResponse::new(PagedResponse::with_entire(vos)))
}

/// Create a custom role
#[utoipa::path(
    post,
    path = "/tenants/{tenant_id}/applications/{application_id}/roles",
    tag = "ApplicationRoles",
    params(
        ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
        ("tenant_id" = String, Path, description = "Tenant ID"),
        ("application_id" = String, Path, description = "Application ID"),
    ),
    request_body = CreateApplicationRoleRequest,
    responses(
        (status = 200, body = ApiResponse<ApplicationRoleVO>),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error"),
    ),
)]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_roles.create",
    skip(_auth, database),
    fields(operator_id = field::Empty, tenant_id = field::Empty, application_id = field::Empty)
)]
pub async fn create_role(
    _auth: AppPermissionGuard<AppUserRead>,
    app: ResolvedApplication,
    State(AppState { database, .. }): State<AppState<'_>>,
    Json(body): Json<CreateApplicationRoleRequest>,
) -> AppResult<ApplicationRoleVO> {
    let application_id = app.id();
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id));
    });

    let role_id = Uuid::now_v7();
    let role = model::application_roles::ActiveModel {
        id: Set(role_id),
        application_id: Set(application_id),
        name: Set(body.name.clone()),
        is_system: Set(false),
    };

    ApplicationRoles::insert(role)
        .exec(&database)
        .await
        .inspect_err(|e| {
            error!(%application_id, error = %e, "failed to create application role");
        })?;

    RolePermissions::set_role_permissions(role_id, &body.permissions, &database)
        .await
        .inspect_err(|e| {
            error!(%application_id, %role_id, error = %e, "failed to set role permissions");
        })?;

    info!(
        %application_id,
        %role_id,
        "application role created successfully"
    );

    Ok(ApiResponse::new(application_role_model_to_vo(
        model::application_roles::Model {
            id: role_id,
            application_id,
            name: body.name,
            is_system: false,
        },
        body.permissions,
    )))
}

/// Get a single role with permissions
#[utoipa::path(
    get,
    path = "/tenants/{tenant_id}/applications/{application_id}/roles/{role_id}",
    tag = "ApplicationRoles",
    params(
        ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
        ("tenant_id" = String, Path, description = "Tenant ID"),
        ("application_id" = String, Path, description = "Application ID"),
        ("role_id" = String, Path, description = "Role ID"),
    ),
    responses(
        (status = 200, body = ApiResponse<ApplicationRoleVO>),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Role not found"),
        (status = 500, description = "Internal server error"),
    ),
)]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_roles.get",
    skip(_auth, database),
    fields(operator_id = field::Empty, tenant_id = field::Empty, application_id = field::Empty, role_id = field::Empty)
)]
pub async fn get_role(
    _auth: AppPermissionGuard<AppUserRead>,
    app: ResolvedApplication,
    State(AppState { database, .. }): State<AppState<'_>>,
    Path((_tenant_id, _application_id, role_id)): Path<(Sqid, Sqid, Sqid)>,
) -> AppResult<ApplicationRoleVO> {
    let application_id = app.id();
    let role_id: Uuid = role_id.try_into()?;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id))
            .record("role_id", field::display(&role_id));
    });

    let role = ApplicationRoles::find_by_id(role_id)
        .one(&database)
        .await
        .inspect_err(|e| {
            error!(%application_id, %role_id, error = %e, "failed to get application role");
        })?
        .ok_or_else(|| Error::with_code(StatusCode::NOT_FOUND, "role not found"))?;

    ensure_belongs_to_app(&role, application_id)?;

    let permissions = RolePermissions::get_role_permissions(role_id, &database)
        .await
        .inspect_err(|e| {
            error!(%role_id, error = %e, "failed to get role permissions");
        })?;

    Ok(ApiResponse::new(application_role_model_to_vo(
        role,
        permissions,
    )))
}

/// Update role name (non-system only)
#[utoipa::path(
    patch,
    path = "/tenants/{tenant_id}/applications/{application_id}/roles/{role_id}",
    tag = "ApplicationRoles",
    params(
        ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
        ("tenant_id" = String, Path, description = "Tenant ID"),
        ("application_id" = String, Path, description = "Application ID"),
        ("role_id" = String, Path, description = "Role ID"),
    ),
    request_body = PatchApplicationRoleRequest,
    responses(
        (status = 200, body = ApiResponse<ApplicationRoleVO>),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - cannot modify system roles"),
        (status = 404, description = "Role not found"),
        (status = 500, description = "Internal server error"),
    ),
)]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_roles.patch",
    skip(_auth, database),
    fields(operator_id = field::Empty, tenant_id = field::Empty, application_id = field::Empty, role_id = field::Empty)
)]
pub async fn patch_role(
    _auth: AppPermissionGuard<AppUserRead>,
    app: ResolvedApplication,
    State(AppState { database, .. }): State<AppState<'_>>,
    Path((_tenant_id, _application_id, role_id)): Path<(Sqid, Sqid, Sqid)>,
    Json(body): Json<PatchApplicationRoleRequest>,
) -> AppResult<ApplicationRoleVO> {
    let application_id = app.id();
    let role_id: Uuid = role_id.try_into()?;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id))
            .record("role_id", field::display(&role_id));
    });

    let role = ApplicationRoles::find_by_id(role_id)
        .one(&database)
        .await
        .inspect_err(|e| {
            error!(%application_id, %role_id, error = %e, "failed to get application role for patch");
        })?
        .ok_or_else(|| {
            Error::with_code(StatusCode::NOT_FOUND, "role not found")
        })?;

    ensure_belongs_to_app(&role, application_id)?;

    if role.is_system {
        return Err(Error::with_code(
            StatusCode::FORBIDDEN,
            "cannot modify system roles",
        ));
    }

    if let Some(name) = body.name {
        let mut active: model::application_roles::ActiveModel = role.clone().into();
        active.name = Set(name);
        ApplicationRoles::update(active)
            .exec(&database)
            .await
            .inspect_err(|e| {
                error!(%application_id, %role_id, error = %e, "failed to update application role");
            })?;
    }

    let updated = ApplicationRoles::find_by_id(role_id)
        .one(&database)
        .await?
        .ok_or_else(|| Error::with_code(StatusCode::NOT_FOUND, "role not found after update"))?;

    let permissions = RolePermissions::get_role_permissions(role_id, &database)
        .await
        .inspect_err(|e| {
            error!(%role_id, error = %e, "failed to get role permissions");
        })?;

    Ok(ApiResponse::new(application_role_model_to_vo(
        updated,
        permissions,
    )))
}

/// Delete role (non-system only)
#[utoipa::path(
    delete,
    path = "/tenants/{tenant_id}/applications/{application_id}/roles/{role_id}",
    tag = "ApplicationRoles",
    params(
        ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
        ("tenant_id" = String, Path, description = "Tenant ID"),
        ("application_id" = String, Path, description = "Application ID"),
        ("role_id" = String, Path, description = "Role ID"),
    ),
    responses(
        (status = 200, description = "Role deleted successfully"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - cannot delete system roles"),
        (status = 404, description = "Role not found"),
        (status = 500, description = "Internal server error"),
    ),
)]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_roles.delete",
    skip(_auth, database),
    fields(operator_id = field::Empty, tenant_id = field::Empty, application_id = field::Empty, role_id = field::Empty)
)]
pub async fn delete_role(
    _auth: AppPermissionGuard<AppUserRead>,
    app: ResolvedApplication,
    State(AppState { database, .. }): State<AppState<'_>>,
    Path((_tenant_id, _application_id, role_id)): Path<(Sqid, Sqid, Sqid)>,
) -> AppResult<()> {
    let application_id = app.id();
    let role_id: Uuid = role_id.try_into()?;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id))
            .record("role_id", field::display(&role_id));
    });

    let role = ApplicationRoles::find_by_id(role_id)
        .one(&database)
        .await
        .inspect_err(|e| {
            error!(%application_id, %role_id, error = %e, "failed to get application role for delete");
        })?
        .ok_or_else(|| {
            Error::with_code(StatusCode::NOT_FOUND, "role not found")
        })?;

    ensure_belongs_to_app(&role, application_id)?;

    if role.is_system {
        return Err(Error::with_code(
            StatusCode::FORBIDDEN,
            "cannot delete system roles",
        ));
    }

    ApplicationRoles::delete_by_id(role_id)
        .exec(&database)
        .await
        .inspect_err(|e| {
            error!(%application_id, %role_id, error = %e, "failed to delete application role");
        })?;

    info!(
        %application_id,
        %role_id,
        "application role deleted successfully"
    );

    Ok(ApiResponse::default())
}

/// Get permissions for a role
#[utoipa::path(
    get,
    path = "/tenants/{tenant_id}/applications/{application_id}/roles/{role_id}/permissions",
    tag = "ApplicationRoles",
    params(
        ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
        ("tenant_id" = String, Path, description = "Tenant ID"),
        ("application_id" = String, Path, description = "Application ID"),
        ("role_id" = String, Path, description = "Role ID"),
    ),
    responses(
        (status = 200, body = ApiResponse<RolePermissionsVO>),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Role not found"),
        (status = 500, description = "Internal server error"),
    ),
)]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_roles.get_permissions",
    skip(_auth, database),
    fields(operator_id = field::Empty, tenant_id = field::Empty, application_id = field::Empty, role_id = field::Empty)
)]
pub async fn get_role_permissions(
    _auth: AppPermissionGuard<AppUserRead>,
    app: ResolvedApplication,
    State(AppState { database, .. }): State<AppState<'_>>,
    Path((_tenant_id, _application_id, role_id)): Path<(Sqid, Sqid, Sqid)>,
) -> AppResult<RolePermissionsVO> {
    let application_id = app.id();
    let role_id: Uuid = role_id.try_into()?;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id))
            .record("role_id", field::display(&role_id));
    });

    let role = ApplicationRoles::find_by_id(role_id)
        .one(&database)
        .await
        .inspect_err(|e| {
            error!(%application_id, %role_id, error = %e, "failed to get application role");
        })?
        .ok_or_else(|| Error::with_code(StatusCode::NOT_FOUND, "role not found"))?;

    ensure_belongs_to_app(&role, application_id)?;

    let permissions = RolePermissions::get_role_permissions(role_id, &database)
        .await
        .inspect_err(|e| {
            error!(%role_id, error = %e, "failed to get role permissions");
        })?;

    Ok(ApiResponse::new(RolePermissionsVO { permissions }))
}

/// Replace all permissions for a role
#[utoipa::path(
    put,
    path = "/tenants/{tenant_id}/applications/{application_id}/roles/{role_id}/permissions",
    tag = "ApplicationRoles",
    params(
        ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
        ("tenant_id" = String, Path, description = "Tenant ID"),
        ("application_id" = String, Path, description = "Application ID"),
        ("role_id" = String, Path, description = "Role ID"),
    ),
    request_body = SetRolePermissionsRequest,
    responses(
        (status = 200, body = ApiResponse<RolePermissionsVO>),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Role not found"),
        (status = 500, description = "Internal server error"),
    ),
)]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_roles.set_permissions",
    skip(_auth, database),
    fields(operator_id = field::Empty, tenant_id = field::Empty, application_id = field::Empty, role_id = field::Empty)
)]
pub async fn set_role_permissions(
    _auth: AppPermissionGuard<AppUserRead>,
    app: ResolvedApplication,
    State(AppState { database, .. }): State<AppState<'_>>,
    Path((_tenant_id, _application_id, role_id)): Path<(Sqid, Sqid, Sqid)>,
    Json(body): Json<SetRolePermissionsRequest>,
) -> AppResult<RolePermissionsVO> {
    let application_id = app.id();
    let role_id: Uuid = role_id.try_into()?;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id))
            .record("role_id", field::display(&role_id));
    });

    let role = ApplicationRoles::find_by_id(role_id)
        .one(&database)
        .await
        .inspect_err(|e| {
            error!(%application_id, %role_id, error = %e, "failed to get application role");
        })?
        .ok_or_else(|| Error::with_code(StatusCode::NOT_FOUND, "role not found"))?;

    ensure_belongs_to_app(&role, application_id)?;

    RolePermissions::set_role_permissions(role_id, &body.permissions, &database)
        .await
        .inspect_err(|e| {
            error!(%role_id, error = %e, "failed to set role permissions");
        })?;

    info!(
        %application_id,
        %role_id,
        "role permissions updated successfully"
    );

    Ok(ApiResponse::new(RolePermissionsVO {
        permissions: body.permissions,
    }))
}

/// Get all roles for a subject
#[utoipa::path(
    get,
    path = "/tenants/{tenant_id}/applications/{application_id}/roles/subjects/{subject_id}/roles",
    tag = "ApplicationRoles",
    params(
        ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
        ("tenant_id" = String, Path, description = "Tenant ID"),
        ("application_id" = String, Path, description = "Application ID"),
        ("subject_id" = String, Path, description = "Subject ID"),
    ),
    responses(
        (status = 200, body = ApiResponse<SubjectRolesVO>),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error"),
    ),
)]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_roles.get_subject_roles",
    skip(_auth, database),
    fields(operator_id = field::Empty, tenant_id = field::Empty, application_id = field::Empty, subject_id = field::Empty)
)]
pub async fn get_subject_roles(
    _auth: AppPermissionGuard<AppUserRead>,
    app: ResolvedApplication,
    State(AppState { database, .. }): State<AppState<'_>>,
    Path((_tenant_id, _application_id, subject_id)): Path<(Sqid, Sqid, Sqid)>,
) -> AppResult<SubjectRolesVO> {
    let application_id = app.id();
    let subject_id: Uuid = subject_id.try_into()?;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id))
            .record("subject_id", field::display(&subject_id));
    });

    ensure_subject_belongs_to_app(subject_id, application_id, &database).await?;

    let role_ids = SubjectRoles::get_subject_role_ids(subject_id, &database)
        .await
        .inspect_err(|e| {
            error!(%application_id, %subject_id, error = %e, "failed to get subject role ids");
        })?;

    if role_ids.is_empty() {
        return Ok(ApiResponse::new(SubjectRolesVO {
            subject_id,
            role_ids: vec![],
        }));
    }

    let roles = ApplicationRoles::find()
        .filter(model::application_roles::Column::Id.is_in(role_ids))
        .filter(model::application_roles::Column::ApplicationId.eq(application_id))
        .all(&database)
        .await
        .inspect_err(|e| {
            error!(%application_id, %subject_id, error = %e, "failed to filter subject roles by application");
        })?;

    Ok(ApiResponse::new(SubjectRolesVO {
        subject_id,
        role_ids: roles.into_iter().map(|r| r.id).collect(),
    }))
}

/// Assign role to subject
#[utoipa::path(
    post,
    path = "/tenants/{tenant_id}/applications/{application_id}/roles/subjects/{subject_id}/roles",
    tag = "ApplicationRoles",
    params(
        ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
        ("tenant_id" = String, Path, description = "Tenant ID"),
        ("application_id" = String, Path, description = "Application ID"),
        ("subject_id" = String, Path, description = "Subject ID"),
    ),
    request_body = AssignRoleRequest,
    responses(
        (status = 200, body = ApiResponse<SubjectRolesVO>),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Role not found"),
        (status = 500, description = "Internal server error"),
    ),
)]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_roles.assign_role",
    skip(_auth, database),
    fields(operator_id = field::Empty, tenant_id = field::Empty, application_id = field::Empty, subject_id = field::Empty, role_id = field::Empty)
)]
pub async fn assign_role(
    _auth: AppPermissionGuard<AppUserRead>,
    app: ResolvedApplication,
    State(AppState { database, .. }): State<AppState<'_>>,
    Path((_tenant_id, _application_id, subject_id)): Path<(Sqid, Sqid, Sqid)>,
    Json(body): Json<AssignRoleRequest>,
) -> AppResult<SubjectRolesVO> {
    let application_id = app.id();
    let subject_id: Uuid = subject_id.try_into()?;
    let role_id = body.role_id;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id))
            .record("subject_id", field::display(&subject_id))
            .record("role_id", field::display(&role_id));
    });

    ensure_subject_belongs_to_app(subject_id, application_id, &database).await?;

    let role = ApplicationRoles::find_by_id(role_id)
        .one(&database)
        .await
        .inspect_err(|e| {
            error!(%application_id, %role_id, error = %e, "failed to get application role for assignment");
        })?
        .ok_or_else(|| {
            Error::with_code(StatusCode::NOT_FOUND, "role not found")
        })?;

    ensure_belongs_to_app(&role, application_id)?;

    SubjectRoles::assign_role(subject_id, role_id, &database)
        .await
        .inspect_err(|e| {
            error!(%application_id, %subject_id, %role_id, error = %e, "failed to assign role to subject");
        })?;

    info!(
        %application_id,
        %subject_id,
        %role_id,
        "role assigned to subject successfully"
    );

    let role_ids = SubjectRoles::get_subject_role_ids(subject_id, &database).await?;

    Ok(ApiResponse::new(SubjectRolesVO {
        subject_id,
        role_ids,
    }))
}

/// Unassign role from subject
#[utoipa::path(
    delete,
    path = "/tenants/{tenant_id}/applications/{application_id}/roles/subjects/{subject_id}/roles/{role_id}",
    tag = "ApplicationRoles",
    params(
        ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
        ("tenant_id" = String, Path, description = "Tenant ID"),
        ("application_id" = String, Path, description = "Application ID"),
        ("subject_id" = String, Path, description = "Subject ID"),
        ("role_id" = String, Path, description = "Role ID"),
    ),
    responses(
        (status = 200, description = "Role unassigned successfully"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal server error"),
    ),
)]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_roles.unassign_role",
    skip(_auth, database),
    fields(operator_id = field::Empty, tenant_id = field::Empty, application_id = field::Empty, subject_id = field::Empty, role_id = field::Empty)
)]
pub async fn unassign_role(
    _auth: AppPermissionGuard<AppUserRead>,
    app: ResolvedApplication,
    State(AppState { database, .. }): State<AppState<'_>>,
    Path((_tenant_id, _application_id, subject_id, role_id)): Path<(Sqid, Sqid, Sqid, Sqid)>,
) -> AppResult<()> {
    let application_id = app.id();
    let subject_id: Uuid = subject_id.try_into()?;
    let role_id: Uuid = role_id.try_into()?;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id))
            .record("subject_id", field::display(&subject_id))
            .record("role_id", field::display(&role_id));
    });

    ensure_subject_belongs_to_app(subject_id, application_id, &database).await?;

    let role = ApplicationRoles::find_by_id(role_id)
        .one(&database)
        .await
        .inspect_err(|e| {
            error!(%application_id, %role_id, error = %e, "failed to get application role for unassignment");
        })?
        .ok_or_else(|| {
            Error::with_code(StatusCode::NOT_FOUND, "role not found")
        })?;

    ensure_belongs_to_app(&role, application_id)?;

    SubjectRoles::unassign_role(subject_id, role_id, &database)
        .await
        .inspect_err(|e| {
            error!(%application_id, %subject_id, %role_id, error = %e, "failed to unassign role from subject");
        })?;

    info!(
        %application_id,
        %subject_id,
        %role_id,
        "role unassigned from subject successfully"
    );

    Ok(ApiResponse::default())
}
