//! Application user-related API endpoints

use axum::{
    Json,
    extract::{Path, State},
};
use axum_valid::Garde;
use oceaniam_audit::types::{AuditPayload, CreateApplicationUserPayload};
use oceaniam_common::{
    ApiResponse, ErrorResponse, PagedResponse, RestResult, jwt::SystemClaim, types::sqid::Sqid,
};
use oceaniam_database::helper::users::{CreateUserOpts, UserHelper};
use oceaniam_database::model::prelude::Users;
use oceaniam_vo::applications::{ApplicationUserVO, CreateApplicationUserRequest};
use tap::Tap;
use tracing::{Span, error, field, info};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    endpoints::application_guards::RequireMatchedApplicationSecret,
    endpoints::applications::{TenantApplicationPath, get_tenant_application},
    middlewares,
    state::{AppState, applications::UserIdentifier},
};

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState<'a>>) -> OpenApiRouter<AppState<'a>> {
    router
        .routes(routes!(get_application_users))
        .routes(routes!(create_application_user))
        .routes(routes!(get_application_user))
}

/// Get application user list
#[utoipa::path(
        get,
        path = "/tenants/{tenant_id}/applications/{application_id}/users",
        tag = "ApplicationUsers",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("application_id" = String, Path, description = "Application ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<PagedResponse<ApplicationUserVO>>),
            (status = 203, description = "Missing Authorization header"),
            (status = 400, description = "Invalid token or bad request", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_users.list",
    skip(auth, database, path),
    fields(operator_id = field::Empty, tenant_id = field::Empty, application_id = field::Empty)
)]
pub async fn get_application_users(
    auth: middlewares::auth::RequireAuth<SystemClaim>,
    State(AppState { database, .. }): State<AppState<'_>>,
    Path(path): Path<TenantApplicationPath>,
) -> RestResult<PagedResponse<ApplicationUserVO>> {
    let operator_id = auth.token.claims.sub;
    let application = get_tenant_application(path, &database).await?;
    let application_id = application.id;
    Span::current().tap(|it| {
        it.record("operator_id", field::display(&operator_id))
            .record("tenant_id", field::display(&application.tenant_id))
            .record("application_id", field::display(&application_id));
    });

    let items: Vec<ApplicationUserVO> =
        Users::get_all_users_of_application(application_id, &database)
            .await
            .inspect_err(|e| {
                error!(
                    %operator_id,
                    %application_id,
                    error = %e,
                    "user list query failed"
                )
            })?
            .into_iter()
            .map(ApplicationUserVO::from)
            .collect();

    Ok(ApiResponse::new(PagedResponse::with_entire(items)))
}

/// Get application user detail
#[utoipa::path(
        get,
        path = "/tenants/{tenant_id}/applications/{application_id}/users/{user_id}",
        tag = "ApplicationUsers",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("application_id" = String, Path, description = "Application ID"),
            ("user_id" = String, Path, description = "User ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<ApplicationUserVO>),
            (status = 400, description = "Bad request", body = ApiResponse<ErrorResponse>),
            (status = 401, description = "Unauthorized"),
            (status = 404, description = "User not found"),
            (status = 500, description = "Internal server error"),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_users.get",
    skip(auth, applications, database),
    fields(operator_id = field::Empty, tenant_id = field::Empty, application_id = field::Empty, user_id = field::Empty)
)]
pub async fn get_application_user(
    auth: middlewares::auth::RequireAuth<SystemClaim>,
    State(AppState {
        applications,
        database,
        ..
    }): State<AppState<'_>>,
    Path((tenant_id, application_id, user_id)): Path<(Sqid, Sqid, Sqid)>,
) -> RestResult<ApplicationUserVO> {
    let operator_id = auth.token.claims.sub;
    let application = get_tenant_application(
        TenantApplicationPath {
            tenant_id,
            application_id,
        },
        &database,
    )
    .await?;
    let application_id = application.id;
    let user_id: Uuid = user_id.try_into()?;

    Span::current().tap(|it| {
        it.record("operator_id", field::display(&operator_id))
            .record("tenant_id", field::display(&application.tenant_id))
            .record("application_id", field::display(&application_id))
            .record("user_id", field::display(&user_id));
    });

    let user = applications
        .get_application_users(application_id)
        .await
        .inspect_err(|e| {
            error!(
                %operator_id,
                %application_id,
                %user_id,
                error = %e,
                "failed to get application users helper"
            )
        })?
        .find_user_by(UserIdentifier::Id(user_id))
        .await
        .inspect_err(|e| {
            error!(
                %operator_id,
                %application_id,
                %user_id,
                error = %e,
                "failed to get application user"
            )
        })?;

    Ok(ApiResponse::new(user.into()))
}

/// Create application user
#[utoipa::path(
        post,
        path = "/tenants/{tenant_id}/applications/{application_id}/users",
        tag = "ApplicationUsers",
        params(
            ("X-OceanIAM-Application-Secret" = String, Header, description = "Application secret"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("application_id" = String, Path, description = "Application ID"),
        ),
        request_body = CreateApplicationUserRequest,
        responses(
            (status = 200, body = ApiResponse<ApplicationUserVO>),
            (status = 400, description = "Bad request"),
            (status = 401, description = "Unauthorized"),
            (status = 403, description = "Forbidden - secret does not belong to this application"),
            (status = 404, description = "Application not found"),
            (status = 500, description = "Internal server error"),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_users.create",
    skip(applications, auditing, path, email, phone, nickname, password, database),
    fields(tenant_id = field::Empty, application_id = field::Empty, user_id = field::Empty)
)]
pub async fn create_application_user(
    _: RequireMatchedApplicationSecret,
    State(AppState {
        applications,
        auditing,
        database,
        ..
    }): State<AppState<'_>>,
    Path(path): Path<TenantApplicationPath>,
    Garde(Json(CreateApplicationUserRequest {
        email,
        phone,
        nickname,
        password,
    })): Garde<Json<CreateApplicationUserRequest>>,
) -> RestResult<ApplicationUserVO> {
    let application = get_tenant_application(path, &database).await?;
    let application_id = application.id;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&application.tenant_id))
            .record("application_id", field::display(&application_id));
    });

    let user = applications
        .get_application_users(application_id)
        .await
        .inspect_err(|e| {
            error!(
                %application_id,
                error = %e,
                "failed to get application users helper"
            )
        })?
        .create_user(
            application_id,
            CreateUserOpts {
                nickname,
                email,
                phone,
            },
            password,
        )
        .await
        .inspect_err(|e| {
            error!(
                %application_id,
                error = %e,
                "application user creation failed"
            )
        })?;
    Span::current().tap(|it| {
        it.record("user_id", field::display(&user.id));
    });

    info!(
        %application_id,
        user_id = %user.id,
        "application user created successfully"
    );

    auditing
        .write(AuditPayload::from(CreateApplicationUserPayload {
            application_id,
            user_id: user.id,
            email: user.email.clone(),
            phone: user.phone.clone(),
            nickname: user.nickname.clone(),
        }))
        .await;

    Ok(ApiResponse::new(user.into()))
}
