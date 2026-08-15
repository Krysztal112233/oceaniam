use argon2::Argon2;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_extra::extract::OptionalQuery;
use axum_valid::Garde;
use oceaniam_api::{ApiResponse, Empty, ErrorResponse, PageParam, PagedResponse};
use oceaniam_audit::types::{
    AuditPayload, CreateApplicationUserPayload, CreateDevAccountPayload,
    DeleteApplicationUserPayload,
};
use oceaniam_common::{helpers::gen_random_name, sqid::Sqid};
use oceaniam_database::{
    helper::users::{CreateUserOpts, PatchUserOpts, UserHelper},
    model::prelude::Users,
};
use oceaniam_vo::applications::{
    ApplicationUserVO, ApplicationUsersListQuery, ApplicationUsersSortOrder,
    CreateApplicationUserRequest, CreatedApplicationUserVO, DevAccountOptions,
    PatchApplicationUserCredentialsRequest, PatchApplicationUserRequest,
    SearchApplicationUsersQuery,
};
use oceaniam_vo::auth::{EnrollTotpResponse, VerifyTotpRequest};
use sea_orm::TransactionTrait;
use tap::Tap;
use tracing::{Span, error, field, info};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::ResolvedApplication;
use crate::{
    error::AppResult,
    middlewares::application::AdminJwtOrApplicationSecretGuard,
    middlewares::auth::AuthenticatedOperator,
    state::{AppState, applications::UserIdentifier},
};

/// Default development-account time-to-live (1 hour) when `ttl_seconds` is omitted.
const DEFAULT_DEV_ACCOUNT_TTL_SECONDS: u64 = 3600;

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router
        .routes(routes!(get_application_users))
        .routes(routes!(search_application_users))
        .routes(routes!(create_application_user))
        .routes(routes!(get_application_user))
        .routes(routes!(patch_application_user))
        .routes(routes!(patch_application_user_credentials))
        .routes(routes!(delete_application_user))
        .routes(routes!(enroll_totp))
        .routes(routes!(verify_totp_enrollment))
        .routes(routes!(remove_totp))
}

/// Get application user list
#[utoipa::path(
        get,
        path = "/tenants/{tenant_id}/applications/{application_id}/users",
        tag = "ApplicationUsers",
        params(
            ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
            ("X-OceanIAM-Application-Secret" = String, Header, description = "Application secret"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("application_id" = String, Path, description = "Application ID"),
            ("page" = Option<u64>, Query, description = "Page number"),
            ("per_page" = Option<u64>, Query, description = "Items per page"),
            ("sort_order" = Option<ApplicationUsersSortOrder>, Query, description = "Sort order by created_at"),
        ),
        responses(
            (status = 200, body = ApiResponse<PagedResponse<ApplicationUserVO>>),
            (status = 203, description = "Missing Authorization header and application secret header"),
            (status = 400, description = "Invalid token or bad request", body = ApiResponse<ErrorResponse>),
            (status = 401, description = "Unauthorized"),
            (status = 403, description = "Forbidden - secret does not belong to this application"),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_users.list",
    skip(_auth, database),
    fields(otel.kind = "internal", operator_id = field::Empty, tenant_id = field::Empty, application_id = field::Empty, page = field::Empty, per_page = field::Empty, sort_order = field::Empty)
)]
pub async fn get_application_users(
    _auth: AdminJwtOrApplicationSecretGuard,
    operator: AuthenticatedOperator,
    State(AppState { database, .. }): State<AppState>,
    app: ResolvedApplication,
    OptionalQuery(query): OptionalQuery<ApplicationUsersListQuery>,
) -> AppResult<PagedResponse<ApplicationUserVO>> {
    let query = query.unwrap_or_default();
    let page = PageParam {
        page: query.page,
        per_page: query.per_page,
    };
    let operator_id = operator.0;
    let application_id = app.id();
    Span::current().tap(|it| {
        it.record("operator_id", field::debug(&operator_id))
            .record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id))
            .record("page", page.page)
            .record("per_page", page.per_page)
            .record(
                "sort_order",
                field::display(match query.sort_order {
                    ApplicationUsersSortOrder::Asc => "asc",
                    ApplicationUsersSortOrder::Desc => "desc",
                }),
            );
    });

    let PagedResponse { items, page_info } = Users::get_users(
        application_id,
        Some(page),
        matches!(query.sort_order, ApplicationUsersSortOrder::Desc),
        &database,
    )
    .await
    .inspect_err(|e| {
        error!(
            ?operator_id,
            %application_id,
            error = %e,
            "user list query failed"
        )
    })?;
    let items = items
        .into_iter()
        .map(crate::conversion::users::user_model_to_vo)
        .collect();

    Ok(ApiResponse::new(PagedResponse { items, page_info }))
}

/// Search application users
///
/// NOTE: Due to system constraints, searching by `by_id` is always an exact match because the value
/// must be converted into a `Uuid` before lookup. Once `by_id` is provided, all other search
/// conditions are ignored because this condition is unique by itself.
#[utoipa::path(
        get,
        path = "/tenants/{tenant_id}/applications/{application_id}/users/search",
        tag = "ApplicationUsers",
        params(
            ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
            ("X-OceanIAM-Application-Secret" = String, Header, description = "Application secret"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("application_id" = String, Path, description = "Application ID"),
            ("by_nickname" = Option<String>, Query, description = "Search users by nickname with fuzzy matching"),
            ("by_email" = Option<String>, Query, description = "Search users by email with fuzzy matching"),
            ("by_phone" = Option<String>, Query, description = "Search users by phone with fuzzy matching"),
            ("by_id" = Option<String>, Query, description = "Search users by id with exact matching; ignores other search conditions once provided"),
            ("page" = Option<u64>, Query, description = "Page number"),
            ("per_page" = Option<u64>, Query, description = "Items per page"),
            ("sort_order" = Option<ApplicationUsersSortOrder>, Query, description = "Sort order by created_at"),
        ),
        responses(
            (status = 200, body = ApiResponse<PagedResponse<ApplicationUserVO>>),
            (status = 203, description = "Missing Authorization header and application secret header"),
            (status = 400, description = "Invalid token or bad request", body = ApiResponse<ErrorResponse>),
            (status = 401, description = "Unauthorized"),
            (status = 403, description = "Forbidden - secret does not belong to this application"),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_users.search",
    skip(_auth, applications, database, search_options),
    fields(otel.kind = "internal", operator_id = field::Empty, tenant_id = field::Empty, application_id = field::Empty, page = field::Empty, per_page = field::Empty, sort_order = field::Empty, by_nickname = field::Empty, by_id = field::Empty, by_email = field::Empty, by_phone = field::Empty)
)]
pub async fn search_application_users(
    _auth: AdminJwtOrApplicationSecretGuard,
    operator: AuthenticatedOperator,
    State(AppState {
        applications,
        database,
        ..
    }): State<AppState>,
    app: ResolvedApplication,
    Garde(Query(search_options)): Garde<Query<SearchApplicationUsersQuery>>,
) -> AppResult<PagedResponse<ApplicationUserVO>> {
    let page = PageParam {
        page: search_options.page,
        per_page: search_options.per_page,
    };
    let sort_desc = matches!(search_options.sort_order, ApplicationUsersSortOrder::Desc);
    let operator_id = operator.0;
    let application_id = app.id();

    if !(search_options
        .by_nickname
        .as_deref()
        .map(str::trim)
        .filter(|it| !it.is_empty())
        .is_some()
        || search_options
            .by_email
            .as_deref()
            .map(str::trim)
            .filter(|it| !it.is_empty())
            .is_some()
        || search_options
            .by_phone
            .as_deref()
            .map(str::trim)
            .filter(|it| !it.is_empty())
            .is_some()
        || search_options.by_id.is_some())
    {
        return Err(crate::error::Error::with_code(
            StatusCode::BAD_REQUEST,
            "at least one of by_nickname, by_email, by_phone or by_id must be provided",
        ));
    };

    Span::current().tap(|it| {
        it.record("operator_id", field::debug(&operator_id))
            .record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id))
            .record("page", page.page)
            .record("per_page", page.per_page)
            .record(
                "sort_order",
                field::display(match search_options.sort_order {
                    ApplicationUsersSortOrder::Asc => "asc",
                    ApplicationUsersSortOrder::Desc => "desc",
                }),
            )
            .record(
                "by_nickname",
                field::display(search_options.by_nickname.as_deref().unwrap_or_default()),
            )
            .record(
                "by_id",
                field::display(
                    search_options
                        .by_id
                        .clone()
                        .map(|it| it.to_string())
                        .unwrap_or_default(),
                ),
            )
            .record(
                "by_email",
                field::display(search_options.by_email.as_deref().unwrap_or_default()),
            )
            .record(
                "by_phone",
                field::display(search_options.by_phone.as_deref().unwrap_or_default()),
            );
    });

    let users = match search_options.by_id {
        Some(by_id) => {
            let user = applications
                .get_application_users(application_id)
                .await
                .inspect_err(|e| {
                    error!(
                        ?operator_id,
                        %application_id,
                        error = %e,
                        "failed to get application users helper"
                    )
                })?
                .find_user_by(UserIdentifier::Id(crate::conversion::sqid::sqid_to_uuid(
                    &by_id,
                )?))
                .await
                .inspect_err(|e| {
                    error!(
                        ?operator_id,
                        %application_id,
                        error = %e,
                        "application user search failed"
                    )
                })?;

            let items = if page.as_offset() == 0 && page.per_page > 0 {
                vec![crate::conversion::users::user_model_to_vo(user)]
            } else {
                Vec::new()
            };

            PagedResponse {
                items,
                page_info: oceaniam_api::PageInfo {
                    has_next: false,
                    total: 1,
                },
            }
        }

        None => {
            let PagedResponse { items, page_info } = Users::search_user(
                application_id,
                search_options.by_nickname,
                search_options.by_email,
                search_options.by_phone,
                page,
                sort_desc,
                &database,
            )
            .await
            .inspect_err(|e| {
                error!(
                    ?operator_id,
                    %application_id,
                    error = %e,
                    "application user search failed"
                )
            })?;

            PagedResponse {
                items: items
                    .into_iter()
                    .map(crate::conversion::users::user_model_to_vo)
                    .collect(),
                page_info,
            }
        }
    };

    Ok(ApiResponse::new(users))
}

/// Get application user detail
#[utoipa::path(
        get,
        path = "/tenants/{tenant_id}/applications/{application_id}/users/{user_id}",
        tag = "ApplicationUsers",
        params(
            ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
            ("X-OceanIAM-Application-Secret" = String, Header, description = "Application secret"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("application_id" = String, Path, description = "Application ID"),
            ("user_id" = String, Path, description = "User ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<ApplicationUserVO>),
            (status = 203, description = "Missing Authorization header and application secret header"),
            (status = 400, description = "Bad request", body = ApiResponse<ErrorResponse>),
            (status = 401, description = "Unauthorized"),
            (status = 403, description = "Forbidden - secret does not belong to this application"),
            (status = 404, description = "User not found"),
            (status = 500, description = "Internal server error"),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_users.get",
    skip(_auth, applications),
    fields(otel.kind = "internal", operator_id = field::Empty, tenant_id = field::Empty, application_id = field::Empty, user_id = field::Empty)
)]
pub async fn get_application_user(
    _auth: AdminJwtOrApplicationSecretGuard,
    operator: AuthenticatedOperator,
    State(AppState { applications, .. }): State<AppState>,
    app: ResolvedApplication,
    Path((_tenant_id, _application_id, user_id)): Path<(Sqid, Sqid, Sqid)>,
) -> AppResult<ApplicationUserVO> {
    let operator_id = operator.0;
    let application_id = app.id();
    let user_id: Uuid = user_id.try_into()?;

    Span::current().tap(|it| {
        it.record("operator_id", field::debug(&operator_id))
            .record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id))
            .record("user_id", field::display(&user_id));
    });

    let user = applications
        .get_application_users(application_id)
        .await
        .inspect_err(|e| {
            error!(
                ?operator_id,
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
                ?operator_id,
                %application_id,
                %user_id,
                error = %e,
                "failed to get application user"
            )
        })?;

    Ok(ApiResponse::new(
        crate::conversion::users::user_model_to_vo(user),
    ))
}

/// Create an application user
///
/// Omitting `development` creates a permanent user. Supplying `development` creates a
/// time-limited development account; an empty object uses the default 3600-second TTL.
#[utoipa::path(
        post,
        path = "/tenants/{tenant_id}/applications/{application_id}/users",
        tag = "ApplicationUsers",
        params(
            ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
            ("X-OceanIAM-Application-Secret" = String, Header, description = "Application secret"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("application_id" = String, Path, description = "Application ID"),
        ),
        request_body = CreateApplicationUserRequest,
        responses(
            (status = 200, body = ApiResponse<CreatedApplicationUserVO>),
            (status = 203, description = "Missing Authorization header and application secret header"),
            (status = 400, description = "Bad request", body = ApiResponse<ErrorResponse>),
            (status = 401, description = "Unauthorized"),
            (status = 403, description = "Forbidden - secret does not belong to this application"),
            (status = 404, description = "Application not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_users.create",
    skip(applications, auditing, email, phone, nickname, password, development, database),
    fields(otel.kind = "internal", tenant_id = field::Empty, application_id = field::Empty, user_id = field::Empty, account_type = field::Empty, ttl_seconds = field::Empty)
)]
pub async fn create_application_user(
    _: AdminJwtOrApplicationSecretGuard,
    State(AppState {
        applications,
        auditing,
        database,
        ..
    }): State<AppState>,
    app: ResolvedApplication,
    Garde(Json(CreateApplicationUserRequest {
        email,
        phone,
        nickname,
        password,
        development,
    })): Garde<Json<CreateApplicationUserRequest>>,
) -> AppResult<CreatedApplicationUserVO> {
    let application_id = app.id();
    let ttl_seconds = development.map(|DevAccountOptions { ttl_seconds }| {
        ttl_seconds.unwrap_or(DEFAULT_DEV_ACCOUNT_TTL_SECONDS)
    });

    // NOTE: This field required more than 4 char if [Some] or [None].
    //
    // If less than 4 char, system will reject request.
    //
    // If [None], it will be filled with [gen_random_name]
    let nickname = nickname.unwrap_or_else(gen_random_name);
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id))
            .record(
                "account_type",
                field::display(if ttl_seconds.is_some() {
                    "development"
                } else {
                    "permanent"
                }),
            )
            .record("ttl_seconds", field::debug(&ttl_seconds));
    });

    let transaction = database.begin().await?;
    let users = applications
        .get_application_users(application_id)
        .await
        .inspect_err(|e| {
            error!(
                %application_id,
                error = %e,
                "failed to get application users helper"
            )
        })?;
    let opts = CreateUserOpts {
        nickname,
        email,
        phone,
    };
    let (user, expires_at) = match ttl_seconds {
        Some(ttl_seconds) => users
            .create_dev_account_in_tx(application_id, opts, password, ttl_seconds, &transaction)
            .await
            .map(|(user, expires_at)| (user, Some(expires_at)))
            .inspect_err(|e| {
                error!(
                    %application_id,
                    error = %e,
                    "development account creation failed"
                )
            })?,
        None => users
            .create_user_in_tx(application_id, opts, password, &transaction)
            .await
            .map(|user| (user, None))
            .inspect_err(|e| {
                error!(
                    %application_id,
                    error = %e,
                    "permanent application user creation failed"
                )
            })?,
    };
    transaction.commit().await?;

    Span::current().tap(|it| {
        it.record("user_id", field::display(&user.id));
    });

    let response = match expires_at {
        Some(expires_at) => {
            info!(
                %application_id,
                user_id = %user.id,
                %expires_at,
                "development account created successfully"
            );

            auditing
                .write(AuditPayload::from(CreateDevAccountPayload {
                    application_id,
                    user_id: user.id,
                    email: user.email.clone(),
                    phone: user.phone.clone(),
                    nickname: user.nickname.clone(),
                    expires_at,
                }))
                .await;

            crate::conversion::users::created_user_model_to_vo(user, Some(expires_at))
        }
        None => {
            info!(
                %application_id,
                user_id = %user.id,
                "permanent application user created successfully"
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

            crate::conversion::users::created_user_model_to_vo(user, None)
        }
    };

    Ok(ApiResponse::new(response))
}

/// Patch application user profile fields (currently nickname only).
///
/// When `nickname` is provided it must be at least 4 characters long.
#[utoipa::path(
        patch,
        path = "/tenants/{tenant_id}/applications/{application_id}/users/{user_id}",
        tag = "ApplicationUsers",
        params(
            ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
            ("X-OceanIAM-Application-Secret" = String, Header, description = "Application secret"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("application_id" = String, Path, description = "Application ID"),
            ("user_id" = String, Path, description = "User ID"),
        ),
        request_body = PatchApplicationUserRequest,
        responses(
            (status = 200, body = ApiResponse<ApplicationUserVO>),
            (status = 203, description = "Missing Authorization header and application secret header"),
            (status = 400, description = "Bad request", body = ApiResponse<ErrorResponse>),
            (status = 401, description = "Unauthorized"),
            (status = 403, description = "Forbidden - secret does not belong to this application"),
            (status = 404, description = "Application or user not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_users.patch",
    skip(_auth, applications),
    fields(otel.kind = "internal", tenant_id = field::Empty, application_id = field::Empty, user_id = field::Empty)
)]
pub async fn patch_application_user(
    _auth: AdminJwtOrApplicationSecretGuard,
    State(AppState { applications, .. }): State<AppState>,
    app: ResolvedApplication,
    Path((_tenant_id, _application_id, user_id)): Path<(Sqid, Sqid, Sqid)>,
    Garde(Json(PatchApplicationUserRequest { nickname })): Garde<Json<PatchApplicationUserRequest>>,
) -> AppResult<ApplicationUserVO> {
    let application_id = app.id();
    let user_id: Uuid = user_id.try_into()?;

    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id))
            .record("user_id", field::display(&user_id));
    });

    let users = applications
        .get_application_users(application_id)
        .await
        .inspect_err(|e| {
            error!(
                %application_id,
                %user_id,
                error = %e,
                "failed to get application users helper"
            )
        })?;

    // TODO: Extend PatchApplicationUserRequest to support patching email and phone.
    let user = users
        .patch_user(
            application_id,
            user_id,
            PatchUserOpts {
                nickname,
                email: None,
                phone: None,
            },
        )
        .await
        .inspect_err(|e| {
            error!(
                %application_id,
                %user_id,
                error = %e,
                "failed to patch application user"
            )
        })?;

    info!(
        %application_id,
        %user_id,
        "application user patched successfully"
    );

    Ok(ApiResponse::new(
        crate::conversion::users::user_model_to_vo(user),
    ))
}

/// Patch application user credentials
/// NOTE: `password`, when provided, must be at least 12 characters long.
#[utoipa::path(
        patch,
        path = "/tenants/{tenant_id}/applications/{application_id}/users/{user_id}/credentials",
        tag = "ApplicationUsers",
        params(
            ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
            ("X-OceanIAM-Application-Secret" = String, Header, description = "Application secret"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("application_id" = String, Path, description = "Application ID"),
            ("user_id" = String, Path, description = "User ID"),
        ),
        request_body = PatchApplicationUserCredentialsRequest,
        responses(
            (status = 200, body = ApiResponse<ApplicationUserVO>),
            (status = 203, description = "Missing Authorization header and application secret header"),
            (status = 400, description = "Bad request", body = ApiResponse<ErrorResponse>),
            (status = 401, description = "Unauthorized"),
            (status = 403, description = "Forbidden - secret does not belong to this application"),
            (status = 404, description = "Application or user not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_users.patch_credentials",
    skip(applications, credentials, database, password),
    fields(otel.kind = "internal", tenant_id = field::Empty, application_id = field::Empty, user_id = field::Empty)
)]
pub async fn patch_application_user_credentials(
    _: AdminJwtOrApplicationSecretGuard,
    State(AppState {
        applications,
        credentials,
        database,
        ..
    }): State<AppState>,
    app: ResolvedApplication,
    Path((_tenant_id, _application_id, user_id)): Path<(Sqid, Sqid, Sqid)>,
    Garde(Json(PatchApplicationUserCredentialsRequest { password })): Garde<
        Json<PatchApplicationUserCredentialsRequest>,
    >,
) -> AppResult<ApplicationUserVO> {
    let application_id = app.id();
    let user_id: Uuid = user_id.try_into()?;

    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id))
            .record("user_id", field::display(&user_id));
    });

    let transaction = database.begin().await?;
    let user = applications
        .get_application_users(application_id)
        .await
        .inspect_err(|e| {
            error!(
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
                %application_id,
                %user_id,
                error = %e,
                "failed to get application user before credential patch"
            )
        })?;

    if let Some(password) = password {
        let argon2 = Argon2::default();
        credentials
            .update_password_in_tx(user_id, password, argon2, &transaction)
            .await
            .inspect_err(|e| {
                error!(
                    %application_id,
                    %user_id,
                    error = %e,
                    "failed to update application user password"
                )
            })?;
    }
    transaction.commit().await?;

    info!(
        %application_id,
        %user_id,
        "application user credentials updated successfully"
    );

    Ok(ApiResponse::new(
        crate::conversion::users::user_model_to_vo(user),
    ))
}

/// Delete application user
#[utoipa::path(
        delete,
        path = "/tenants/{tenant_id}/applications/{application_id}/users/{user_id}",
        tag = "ApplicationUsers",
        params(
            ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
            ("X-OceanIAM-Application-Secret" = String, Header, description = "Application secret"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("application_id" = String, Path, description = "Application ID"),
            ("user_id" = String, Path, description = "User ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<Empty>),
            (status = 203, description = "Missing Authorization header and application secret header"),
            (status = 400, description = "Bad request", body = ApiResponse<ErrorResponse>),
            (status = 401, description = "Unauthorized"),
            (status = 403, description = "Forbidden - secret does not belong to this application"),
            (status = 404, description = "Application or user not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_users.delete",
    skip(_auth, applications, auditing, database),
    fields(otel.kind = "internal", tenant_id = field::Empty, application_id = field::Empty, user_id = field::Empty)
)]
pub async fn delete_application_user(
    _auth: AdminJwtOrApplicationSecretGuard,
    State(AppState {
        applications,
        auditing,
        database,
        ..
    }): State<AppState>,
    app: ResolvedApplication,
    Path((_tenant_id, _application_id, user_id)): Path<(Sqid, Sqid, Sqid)>,
) -> AppResult<Empty> {
    let application_id = app.id();
    let user_id: Uuid = user_id.try_into()?;

    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id))
            .record("user_id", field::display(&user_id));
    });

    // Verify the user exists and belongs to this application (returns 404 otherwise).
    applications
        .get_application_users(application_id)
        .await
        .inspect_err(|e| {
            error!(
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
                %application_id,
                %user_id,
                error = %e,
                "failed to get application user before delete"
            )
        })?;

    let transaction = database.begin().await?;
    applications
        .get_application_users(application_id)
        .await
        .inspect_err(|e| {
            error!(
                %application_id,
                %user_id,
                error = %e,
                "failed to get application users helper"
            )
        })?
        .delete_user_in_tx(application_id, user_id, &transaction)
        .await?;
    transaction.commit().await?;

    info!(%application_id, %user_id, "application user deleted successfully");

    auditing
        .write(AuditPayload::from(DeleteApplicationUserPayload {
            application_id,
            user_id,
        }))
        .await;

    Ok(ApiResponse::empty())
}

/// Enroll in TOTP multi-factor authentication
///
/// Generates a TOTP secret and returns a provisioning URI for the user to scan with their
/// authenticator app. The secret is cached temporarily — call the verify endpoint with the
/// TOTP code shown in the authenticator app to complete enrollment.
#[utoipa::path(
    post,
    path = "/tenants/{tenant_id}/applications/{application_id}/users/{user_id}/totp/enroll",
    tag = "ApplicationUsers",
    params(
        ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
        ("X-OceanIAM-Application-Secret" = String, Header, description = "Application secret"),
        ("tenant_id" = String, Path, description = "Tenant ID"),
        ("application_id" = String, Path, description = "Application ID"),
        ("user_id" = String, Path, description = "User ID"),
    ),
    responses(
        (status = 200, body = ApiResponse<EnrollTotpResponse>),
        (status = 203, description = "Missing Authorization header and application secret header"),
        (status = 400, description = "Bad request", body = ApiResponse<ErrorResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - secret does not belong to this application"),
        (status = 404, description = "Application or user not found", body = ApiResponse<ErrorResponse>),
        (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
    ),
)]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_users.totp_enroll",
    skip(_auth, applications, credentials, app),
    fields(otel.kind = "internal", tenant_id = field::Empty, application_id = field::Empty, user_id = field::Empty)
)]
pub async fn enroll_totp(
    _auth: AdminJwtOrApplicationSecretGuard,
    State(AppState {
        applications,
        credentials,
        ..
    }): State<AppState>,
    app: ResolvedApplication,
    Path((_tenant_id, _application_id, user_id)): Path<(Sqid, Sqid, Sqid)>,
) -> AppResult<EnrollTotpResponse> {
    let application_id = app.id();
    let user_id: Uuid = user_id.try_into()?;

    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id))
            .record("user_id", field::display(&user_id));
    });

    let user = applications
        .get_application_users(application_id)
        .await?
        .find_user_by(UserIdentifier::Id(user_id))
        .await?;

    let config = applications.get_configuration(application_id).await?;
    let issuer = config.auth.token.issuer.clone();
    let account_name = user.email.clone().unwrap_or_else(|| user_id.to_string());

    let response = credentials
        .initiate_totp_enrollment(user_id, &issuer, &account_name)
        .await?;

    info!(%application_id, %user_id, "TOTP enrollment initiated");

    Ok(ApiResponse::new(response))
}

/// Verify TOTP enrollment code
///
/// Completes TOTP enrollment by verifying the code from the user's authenticator app against the
/// previously generated secret. Once verified, TOTP is enabled for this user and will be required
/// during sign-in.
#[utoipa::path(
    post,
    path = "/tenants/{tenant_id}/applications/{application_id}/users/{user_id}/totp/verify",
    tag = "ApplicationUsers",
    params(
        ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
        ("X-OceanIAM-Application-Secret" = String, Header, description = "Application secret"),
        ("tenant_id" = String, Path, description = "Tenant ID"),
        ("application_id" = String, Path, description = "Application ID"),
        ("user_id" = String, Path, description = "User ID"),
    ),
    request_body = VerifyTotpRequest,
    responses(
        (status = 200, body = ApiResponse<Empty>),
        (status = 203, description = "Missing Authorization header and application secret header"),
        (status = 400, description = "Bad request or invalid code", body = ApiResponse<ErrorResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - secret does not belong to this application"),
        (status = 404, description = "Application or user not found", body = ApiResponse<ErrorResponse>),
        (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
    ),
)]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_users.totp_verify",
    skip(_auth, credentials, body),
    fields(otel.kind = "internal", tenant_id = field::Empty, application_id = field::Empty, user_id = field::Empty)
)]
pub async fn verify_totp_enrollment(
    _auth: AdminJwtOrApplicationSecretGuard,
    State(AppState { credentials, .. }): State<AppState>,
    app: ResolvedApplication,
    Path((_tenant_id, _application_id, user_id)): Path<(Sqid, Sqid, Sqid)>,
    Json(body): Json<VerifyTotpRequest>,
) -> AppResult<Empty> {
    let application_id = app.id();
    let user_id: Uuid = user_id.try_into()?;

    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id))
            .record("user_id", field::display(&user_id));
    });

    credentials
        .verify_totp_enrollment(user_id, &body.code)
        .await?;

    info!(%application_id, %user_id, "TOTP enrollment completed");

    Ok(ApiResponse::empty())
}

/// Disable TOTP multi-factor authentication
///
/// Removes the TOTP secret from the user's credentials. After this, MFA will no longer be required
/// during sign-in.
#[utoipa::path(
    delete,
    path = "/tenants/{tenant_id}/applications/{application_id}/users/{user_id}/totp",
    tag = "ApplicationUsers",
    params(
        ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
        ("X-OceanIAM-Application-Secret" = String, Header, description = "Application secret"),
        ("tenant_id" = String, Path, description = "Tenant ID"),
        ("application_id" = String, Path, description = "Application ID"),
        ("user_id" = String, Path, description = "User ID"),
    ),
    responses(
        (status = 200, body = ApiResponse<Empty>),
        (status = 203, description = "Missing Authorization header and application secret header"),
        (status = 400, description = "Bad request", body = ApiResponse<ErrorResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - secret does not belong to this application"),
        (status = 404, description = "Application or user not found", body = ApiResponse<ErrorResponse>),
        (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
    ),
)]
#[tracing::instrument(
    level = "info",
    name = "tenant_application_users.totp_remove",
    skip(_auth, credentials),
    fields(otel.kind = "internal", tenant_id = field::Empty, application_id = field::Empty, user_id = field::Empty)
)]
pub async fn remove_totp(
    _auth: AdminJwtOrApplicationSecretGuard,
    State(AppState { credentials, .. }): State<AppState>,
    app: ResolvedApplication,
    Path((_tenant_id, _application_id, user_id)): Path<(Sqid, Sqid, Sqid)>,
) -> AppResult<Empty> {
    let application_id = app.id();
    let user_id: Uuid = user_id.try_into()?;

    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id))
            .record("user_id", field::display(&user_id));
    });

    credentials.remove_totp(user_id).await?;

    info!(%application_id, %user_id, "TOTP disabled");

    Ok(ApiResponse::empty())
}
