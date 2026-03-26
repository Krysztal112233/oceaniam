//! Application management-related API endpoints
//!
//! Provides interfaces for:
//! - Application CRUD and listing
//! - JWKS retrieval for applications
//! - Application user management and legacy authentication flows
//! - Application secret management

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_extra::extract::cookie::Cookie;
use axum_valid::Garde;
use chrono::Utc;
use itertools::Itertools;
use oceaniam_audit::types::{
    AuditPayload, CreateApplicationPayload, CreateApplicationUserPayload, DeleteApplicationPayload,
    PatchApplicationConfigurationPayload, PatchApplicationPayload, RefreshJwtPayload,
    RevokeJwtPayload, SignJwtPayload,
};
use oceaniam_common::{
    ApiResponse, ApiResponseWithHeader, Empty, ErrorResponse, PagedResponse, RestResult,
    WithHeaderRestResult, consts,
    error::Error,
    jwks::{JwkSet, JwkSetSchema},
    jwt::{Claim, SystemClaim},
    types::sqid::Sqid,
};
use oceaniam_credential::credential::Password;
use oceaniam_database::{
    helper::{
        applications::{ApplicationConfiguration, ApplicationHelper},
        users::{CreateUserOpts, UserHelper},
    },
    model::{self, prelude::*},
};
use oceaniam_keybox::keybox::KeyOption;
use oceaniam_vo::{
    applications::{
        ApplicationConfigurationVO, ApplicationDetailVO, ApplicationUserVO, ApplicationVO,
        CreateApplicationRequest, CreateApplicationResponse, CreateApplicationUserRequest,
        GetApplicationConfigurationResponse, GetApplicationParam,
        PatchApplicationConfigurationRequest, PatchApplicationRequest,
    },
    auth::{AuthVO, SigninResponse, SignoutResponse},
};
use tap::Tap;
use tracing::{Span, error, field, info, warn};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    endpoints::applications::spec_middlewares::RequireMatchedApplicationSecret,
    middlewares::{
        self,
        auth::{RequireAuth, TokenDispatchMethod},
    },
    state::{
        AppState,
        keybox::{EncodedJwt, SignJwtOptions},
    },
};

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState<'a>>) -> OpenApiRouter<AppState<'a>> {
    router
        .routes(routes!(create_application))
        .routes(routes!(get_application))
        .routes(routes!(patch_application))
        .routes(routes!(delete_application))
        .routes(routes!(get_application_configuration))
        .routes(routes!(patch_application_configuration))
        .routes(routes!(get_application_jwks))
        .routes(routes!(get_applications))
        .routes(routes!(get_application_users))
        .routes(routes!(create_application_user))
        .routes(routes!(legacy_create_application_auth_token))
        .routes(routes!(legacy_delete_application_auth_token))
        .routes(routes!(legacy_refresh_application_auth_token))
}

/// Get application list
#[utoipa::path(
        get,
        path = "/applications",
        tag = "Applications",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
        ),
        responses(
            (status = 200, body = ApiResponse<PagedResponse<ApplicationVO>>),
            (status = 203, description = "Missing Authorization header"),
            (status = 400, description = "Invalid token or bad request", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "applications.list",
    skip(database),
    fields(tenant_id = field::Empty, has_pagination = field::Empty)
)]
pub async fn get_applications(
    _: middlewares::auth::RequireAuth<SystemClaim>,
    Query(GetApplicationParam { tenant_id, page }): Query<GetApplicationParam>,
    State(AppState { database, .. }): State<AppState<'_>>,
) -> RestResult<PagedResponse<ApplicationVO>> {
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&tenant_id))
            .record("has_pagination", page.is_some());
    });

    info!(tenant_id = %tenant_id, "getting applications");

    let PagedResponse { items, page_info } = match page {
        Some(page) => {
            Applications::get_applications(tenant_id.try_into()?, page, &database).await?
        }
        None => PagedResponse::with_entire(
            Applications::get_all_applications(tenant_id.try_into()?, &database).await?,
        ),
    };

    Ok(ApiResponse::new(PagedResponse {
        items: items.into_iter().map(ApplicationVO::from).collect(),
        page_info,
    }))
}

/// Create new application
///
/// Creates a new application with an automatically generated RSA key pair
#[utoipa::path(
        post,
        path = "/applications",
        tag = "Applications",
        params(("Authorization" = String, Header, description = "Bearer token")),
        request_body = CreateApplicationRequest,
        responses(
            (status = 200, body = ApiResponse<CreateApplicationResponse>),
            (status = 203, description = "Missing Authorization header"),
            (status = 400, description = "Invalid token or bad request", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "applications.create",
    skip(applications, auditing, keyboxes, comment),
    fields(tenant_id = field::Empty, application_id = field::Empty)
)]
pub async fn create_application(
    _: middlewares::auth::RequireAuth<SystemClaim>,

    State(AppState {
        applications,
        auditing,
        keyboxes,
        ..
    }): State<AppState<'_>>,

    Json(CreateApplicationRequest { tenant_id, comment }): Json<CreateApplicationRequest>,
) -> RestResult<CreateApplicationResponse> {
    let model::applications::Model {
        id,
        comment,
        tenant_id,
        ..
    } = applications
        .create_application(tenant_id.try_into()?, comment)
        .await?;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&tenant_id))
            .record("application_id", field::display(&id));
    });

    info!(
        tenant_id = %tenant_id,
        application_id = %id,
        "application created successfully"
    );

    keyboxes
        .create_keybox(
            id,
            KeyOption {
                retired_at: Some((Utc::now() + consts::DEFAULT_KEY_RETIED_AFTER).into()),
                expires_at: Some((Utc::now() + consts::DEFAULT_KEY_EXPIRES_AFTER).into()),
                ..Default::default()
            },
        )
        .await?;

    info!(
        tenant_id = %tenant_id,
        application_id = %id,
        "default keybox of application created successfully"
    );

    auditing
        .write(AuditPayload::from(CreateApplicationPayload {
            application_id: id,
            tenant_id,
            comment: comment.clone(),
        }))
        .await;

    Ok(ApiResponse::new(CreateApplicationResponse {
        tenant_id: tenant_id.into(),
        application_id: id.into(),
        comment,
    }))
}

/// Get application detail
#[utoipa::path(
        get,
        path = "/applications/{application_id}",
        tag = "Applications",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("application_id" = String, Path, description = "Application ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<ApplicationDetailVO>),
            (status = 400, description = "Invalid application id", body = ApiResponse<ErrorResponse>),
            (status = 404, description = "Application not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "applications.get",
    skip(database, application_id),
    fields(application_id = field::Empty)
)]
pub async fn get_application(
    _: middlewares::auth::RequireAuth<SystemClaim>,

    Path(application_id): Path<Sqid>,
    State(AppState { database, .. }): State<AppState<'_>>,
) -> RestResult<ApplicationDetailVO> {
    let application_id: Uuid = application_id
        .try_into()
        .inspect_err(|e| error!(error = %e, "failed to convert application_id"))?;
    Span::current().tap(|it| {
        it.record("application_id", field::display(&application_id));
    });

    let application = Applications::get_application(application_id, &database)
        .await
        .inspect_err(|e| {
            error!(
                %application_id,
                error = %e,
                "failed to get application detail"
            )
        })?;

    Ok(ApiResponse::new(ApplicationDetailVO::from(application)))
}

/// Patch application
///
/// Partially updates mutable application fields such as `comment`
#[utoipa::path(
        patch,
        path = "/applications/{application_id}",
        tag = "Applications",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("application_id" = String, Path, description = "Application ID"),
        ),
        request_body = PatchApplicationRequest,
        responses(
            (status = 200, body = ApiResponse<ApplicationDetailVO>),
            (status = 400, description = "Invalid application id", body = ApiResponse<ErrorResponse>),
            (status = 404, description = "Application not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "applications.patch",
    skip(applications, auditing, application_id, patch),
    fields(application_id = field::Empty)
)]
pub async fn patch_application(
    _: middlewares::auth::RequireAuth<SystemClaim>,

    State(AppState {
        applications,
        auditing,
        ..
    }): State<AppState<'_>>,

    Path(application_id): Path<Sqid>,
    Json(patch): Json<PatchApplicationRequest>,
) -> RestResult<ApplicationDetailVO> {
    let application_id: Uuid = application_id
        .try_into()
        .inspect_err(|e| error!(error = %e, "failed to convert application_id"))?;
    Span::current().tap(|it| {
        it.record("application_id", field::display(&application_id));
    });

    let application = applications
        .patch_application(application_id, patch)
        .await
        .inspect_err(|e| error!(%application_id, error = %e, "application patch failed"))?;

    auditing
        .write(AuditPayload::from(PatchApplicationPayload {
            application_id,
            comment: application.comment.clone(),
        }))
        .await;

    Ok(ApiResponse::new(ApplicationDetailVO::from(application)))
}

/// Delete application
///
/// Permanently removes an application and all associated data
#[utoipa::path(
        delete,
        path = "/applications/{application_id}",
        tag = "Applications",
        responses(
            (status = 200, body = ApiResponse<Empty>),
            (status = 400, description = "Invalid application id", body = ApiResponse<ErrorResponse>),
            (status = 404, description = "Application not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "applications.delete",
    skip(applications, auditing, application_id),
    fields(application_id = field::Empty)
)]
pub async fn delete_application(
    Path(application_id): Path<Sqid>,

    State(AppState {
        applications,
        auditing,
        ..
    }): State<AppState<'_>>,
) -> RestResult<()> {
    let application_id: Uuid = application_id
        .try_into()
        .inspect_err(|e| error!(error = %e, "failed to convert application_id"))?;
    Span::current().tap(|it| {
        it.record("application_id", field::display(&application_id));
    });

    applications
        .delete_application(application_id)
        .await
        .inspect_err(|e| error!(%application_id, error = %e, "application deletion failed"))?;

    auditing
        .write(AuditPayload::from(DeleteApplicationPayload {
            application_id,
        }))
        .await;

    Ok(ApiResponse::new(()))
}

/// Get application configuration
#[utoipa::path(
        get,
        path = "/applications/{application_id}/configuration",
        tag = "Applications",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("application_id" = String, Path, description = "Application ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<GetApplicationConfigurationResponse>),
            (status = 400, description = "Invalid application id", body = ApiResponse<ErrorResponse>),
            (status = 404, description = "Application not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "applications.configuration",
    skip(applications, application_id),
    fields(application_id = field::Empty)
)]
pub async fn get_application_configuration(
    _: middlewares::auth::RequireAuth<SystemClaim>,

    State(AppState { applications, .. }): State<AppState<'_>>,

    Path(application_id): Path<Sqid>,
) -> RestResult<GetApplicationConfigurationResponse> {
    let application_id: Uuid = application_id
        .try_into()
        .inspect_err(|e| error!(error = %e, "failed to convert application_id"))?;
    Span::current().tap(|it| {
        it.record("application_id", field::display(&application_id));
    });

    let configuration = applications
        .get_configuration(application_id)
        .await
        .inspect_err(|e| {
            error!(
                %application_id,
                error = %e,
                "failed to get application configuration"
            )
        })?;

    Ok(ApiResponse::new(GetApplicationConfigurationResponse {
        configuration: ApplicationConfigurationVO::from(configuration),
    }))
}

/// Patch application configuration
#[utoipa::path(
        patch,
        path = "/applications/{application_id}/configuration",
        tag = "Applications",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("application_id" = String, Path, description = "Application ID"),
        ),
        request_body = PatchApplicationConfigurationRequest,
        responses(
            (status = 200, body = ApiResponse<Empty>),
            (status = 400, description = "Invalid application id", body = ApiResponse<ErrorResponse>),
            (status = 404, description = "Application not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "applications.configuration.patch",
    skip(applications, auditing, application_id, patch),
    fields(application_id = field::Empty)
)]
pub async fn patch_application_configuration(
    _: middlewares::auth::RequireAuth<SystemClaim>,

    State(AppState {
        applications,
        auditing,
        ..
    }): State<AppState<'_>>,

    Path(application_id): Path<Sqid>,
    Json(patch): Json<PatchApplicationConfigurationRequest>,
) -> RestResult<Empty> {
    let application_id: Uuid = application_id
        .try_into()
        .inspect_err(|e| error!(error = %e, "failed to convert application_id"))?;
    Span::current().tap(|it| {
        it.record("application_id", field::display(&application_id));
    });

    applications
        .patch_configuration(application_id, patch)
        .await
        .inspect_err(|e| {
            error!(
                %application_id,
                error = %e,
                "failed to patch application configuration"
            )
        })?;

    auditing
        .write(AuditPayload::from(PatchApplicationConfigurationPayload {
            application_id,
        }))
        .await;

    Ok(ApiResponse::new(Empty::default()))
}

/// Get application JWKS
///
/// Returns the JSON Web Key Set for verifying JWTs issued by this application
#[utoipa::path(
        get,
        path = "/applications/{application_id}/.well-known/jwks.json",
        tag = "Applications",
        responses(
            (status = 200, body = JwkSetSchema),
            (status = 400, description = "Invalid application id", body = ApiResponse<ErrorResponse>),
            (status = 404, description = "Application not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "applications.jwks",
    skip(keyboxes, application_id),
    fields(application_id = field::Empty)
)]
pub async fn get_application_jwks(
    Path(application_id): Path<Sqid>,

    State(AppState { keyboxes, .. }): State<AppState<'_>>,
) -> RestResult<JwkSet> {
    let application_id: Uuid = application_id
        .try_into()
        .inspect_err(|e| error!(error = %e, "failed to convert application_id"))?;
    Span::current().tap(|it| {
        it.record("application_id", field::display(&application_id));
    });

    Ok(ApiResponse::new(
        keyboxes
            .get_jwks(application_id)
            .await
            .inspect_err(|e| error!(%application_id, error = %e, "failed to get jwks"))?,
    ))
}

/// Get user list
#[utoipa::path(
        get,
        path = "/applications/{application_id}/users",
        tag = "ApplicationUsers",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
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
    name = "application_users.list",
    skip(auth, database, application_id),
    fields(operator_id = field::Empty, application_id = field::Empty)
)]
pub async fn get_application_users(
    auth: middlewares::auth::RequireAuth<SystemClaim>,
    State(AppState { database, .. }): State<AppState<'_>>,

    Path(application_id): Path<Sqid>,
) -> RestResult<PagedResponse<ApplicationUserVO>> {
    let operator_id = auth.token.claims.sub;

    let application_id = application_id.try_into()?;
    Span::current().tap(|it| {
        it.record("operator_id", field::display(&operator_id))
            .record("application_id", field::display(&application_id));
    });

    let items = Users::get_all_users_of_application(application_id, &database)
        .await
        .inspect_err(|e| {
            error!(
                operator_id = %operator_id,
                %application_id,
                error = %e,
                "user list query failed"
            )
        })?
        .into_iter()
        .map(Into::into)
        .collect_vec();

    Ok(ApiResponse::new(PagedResponse::with_entire(items)))
}

/// Create application user (signup)
#[utoipa::path(
        post,
        path = "/applications/{application_id}/users",
        tag = "ApplicationUsers",
        params(
            ("X-OceanIAM-Application-Secret" = String, Header, description = "Application secret"),
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
    name = "application_users.create",
    skip(applications, auditing, application_id, email, phone, nickname, password),
    fields(application_id = field::Empty, user_id = field::Empty)
)]
pub async fn create_application_user(
    _: RequireMatchedApplicationSecret,

    State(AppState {
        applications,
        auditing,
        ..
    }): State<AppState<'_>>,

    Path(application_id): Path<Sqid>,
    Garde(Json(CreateApplicationUserRequest {
        email,
        phone,
        nickname,
        password,
    })): Garde<Json<CreateApplicationUserRequest>>,
) -> RestResult<ApplicationUserVO> {
    let application_id: Uuid = application_id
        .try_into()
        .inspect_err(|e| error!(error = %e, "failed to convert application_id"))?;
    Span::current().tap(|it| {
        it.record("application_id", field::display(&application_id));
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

/// Create auth token (signin)
///
/// Issues an application user JWT after verifying the provided credentials.
///
/// The response payload and/or cookie can be controlled via the optional
/// `X-OceanIAM-Token-Dispatch` header:
///
/// - `json`: JSON body only
/// - `cookie`: cookie only (JSON body will be empty: `{}`)
/// - `both`: JSON body + cookie (default)
///
/// Cookie name: `auth_token`.
#[utoipa::path(
        post,
        path = "/applications/{application_id}/auth/tokens",
        tag = "ApplicationUserAuthentication",
        params(
            ("X-OceanIAM-Application-Secret" = String, Header, description = "Application secret"),
            ("X-OceanIAM-Token-Dispatch" = String, Header, description = "Optional token dispatch method. Values: cookie|json|both (case-insensitive; whitespace ignored). Defaults to both."),
            ("application_id" = String, Path, description = "Application ID"),
        ),
        request_body = AuthVO,
        responses(
            (status = 200, body = ApiResponse<Option<SigninResponse>>),
            (status = 400, description = "Bad request"),
            (status = 401, description = "Unauthorized"),
            (status = 403, description = "Forbidden - secret does not belong to this application"),
            (status = 404, description = "Application not found"),
            (status = 500, description = "Internal server error"),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "application_auth.signin_legacy",
    skip(token_mtd, applications, credentials, keyboxes, auditing, application_id, auth),
    fields(
        application_id = field::Empty,
        user_id = field::Empty,
        token_dispatch = field::Empty
    )
)]
pub async fn legacy_create_application_auth_token(
    _: RequireMatchedApplicationSecret,

    token_mtd: TokenDispatchMethod,

    State(AppState {
        applications,
        credentials,
        keyboxes,
        auditing,
        ..
    }): State<AppState<'_>>,

    Path(application_id): Path<Sqid>,
    Json(auth): Json<AuthVO>,
) -> WithHeaderRestResult<Option<SigninResponse>> {
    let application_id: Uuid = application_id
        .try_into()
        .inspect_err(|e| error!(error = %e, "failed to convert application_id"))?;
    Span::current().tap(|it| {
        it.record("application_id", field::display(&application_id))
            .record("token_dispatch", field::debug(&token_mtd));
    });

    let ApplicationConfiguration { authentication, .. } = applications
        .get_configuration(application_id)
        .await
        .inspect_err(|e| {
            error!(
                %application_id,
                error = %e,
                "failed to get application configuration"
            )
        })?;
    let user = applications
        .find_user_by(application_id, auth.clone())
        .await
        .inspect_err(|e| {
            error!(
                %application_id,
                error = %e,
                "failed to find application user"
            )
        })?;
    Span::current().tap(|it| {
        it.record("user_id", field::display(&user.id));
    });

    let vault = credentials.get_credential(user.id).await.inspect_err(|e| {
        error!(
            %application_id,
            user_id = %user.id,
            error = %e,
            "failed to get user credential"
        )
    })?;

    let verify_result = match auth {
        AuthVO::Email { password, .. } | AuthVO::Phone { password, .. } => Password::from(vault)
            .verify(&password)
            .await
            .inspect_err(|e| {
                error!(
                    %application_id,
                    user_id = %user.id,
                    error = %e,
                    "failed to verify password"
                )
            })?,
    };

    if !verify_result {
        return Err(Error::with_code(
            StatusCode::INTERNAL_SERVER_ERROR,
            consts::USER_LOGIN_FAILED_MSG,
        ));
    }

    let EncodedJwt { jwt, claim } = keyboxes
        .sign_jwt::<Claim>(
            user.id,
            SignJwtOptions {
                application_id: user.application_id,
                iss: authentication.issuer,
                aud: authentication.audience,
            },
        )
        .await
        .inspect_err(|e| {
            error!(
                %application_id,
                user_id = %user.id,
                error = %e,
                "failed to sign jwt during legacy signin"
            )
        })?;

    info!(
        %application_id,
        user_id = %user.id,
        "legacy signin successful"
    );

    auditing
        .write(AuditPayload::from(SignJwtPayload {
            application_id: user.application_id,
            subject_id: user.id,
            jti: claim.jti,
        }))
        .await;

    let cookie = Cookie::new("auth_token", jwt.clone());
    let resp = ApiResponseWithHeader::new(Some(SigninResponse { jwt }));

    let resp = match token_mtd {
        TokenDispatchMethod::Cookie => ApiResponseWithHeader::new(None).with_cookie(cookie)?,
        TokenDispatchMethod::Json => resp,
        TokenDispatchMethod::Both => resp.with_cookie(cookie)?,
    };

    Ok(resp)
}

/// Delete auth token (signout)
///
/// Revokes the current JWT by adding its JTI (JWT ID) to the revoked tokens list.
///
/// # Authorization
///
/// Requires `Authorization: Bearer <jwt>`.
#[utoipa::path(
        delete,
        path = "/applications/{application_id}/auth/tokens",
        tag = "ApplicationUserAuthentication",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("X-OceanIAM-Application-Secret" = String, Header, description = "Application secret"),
            ("application_id" = String, Path, description = "Application ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<SignoutResponse>),
            (status = 203, description = "Missing Authorization header"),
            (status = 400, description = "Invalid, expired, or revoked token", body = ApiResponse<ErrorResponse>),
            (status = 401, description = "Unauthorized"),
            (status = 403, description = "Forbidden - secret does not belong to this application"),
            (status = 404, description = "Application not found"),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "application_auth.signout_legacy",
    skip(auth, revoked_jwt, auditing, application_id),
    fields(user_id = field::Empty, application_id = field::Empty, jti = field::Empty)
)]
pub async fn legacy_delete_application_auth_token(
    _: RequireMatchedApplicationSecret,
    auth: RequireAuth<Claim>,

    State(AppState {
        revoked_jwt,
        auditing,
        ..
    }): State<AppState<'_>>,

    Path(application_id): Path<Sqid>,
) -> RestResult<SignoutResponse> {
    // TODO: might need more security...?
    let jti = auth.token.claims.jti;
    let user_id = auth.token.claims.sub;
    let app_id: Uuid = application_id.try_into().inspect_err(|e| {
        error!(
            %user_id,
            %jti,
            error = %e,
            "failed to convert application_id"
        )
    })?;
    Span::current().tap(|it| {
        it.record("user_id", field::display(&user_id))
            .record("application_id", field::display(&app_id))
            .record("jti", field::display(&jti));
    });

    info!(
        %user_id,
        application_id = %app_id,
        %jti,
        "legacy signout requested"
    );

    revoked_jwt.set_revoked(jti).await.inspect_err(|e| {
        error!(
            %user_id,
            application_id = %app_id,
            %jti,
            error = %e,
            "failed to revoke jwt during legacy signout"
        )
    })?;

    info!(
        %user_id,
        application_id = %app_id,
        %jti,
        "legacy signout successful"
    );

    auditing
        .write(AuditPayload::from(RevokeJwtPayload {
            subject_id: user_id,
            jti,
            application_id: Some(app_id),
        }))
        .await;

    Ok(ApiResponse::new(SignoutResponse::default()))
}

/// Refresh auth token
///
/// Rotates the current JWT by revoking its JTI and issuing a new token.
///
/// The new token can be delivered back to the client via JSON response body and/or
/// an HTTP cookie, controlled by the `X-OceanIAM-Token-Dispatch` request header:
///
/// - `json`: JSON body only
/// - `cookie`: cookie only (JSON body will be empty: `{}`)
/// - `both`: JSON body + cookie (default)
///
/// Cookie name: `auth_token`.
///
/// # Authorization
///
/// Requires `Authorization: Bearer <jwt>`.
#[utoipa::path(
        post,
        path = "/applications/{application_id}/auth/tokens/refresh",
        tag = "ApplicationUserAuthentication",
        params(
            ("Authorization" = String, Header, description = "Bearer token to refresh"),
            ("X-OceanIAM-Application-Secret" = String, Header, description = "Application secret"),
            ("X-OceanIAM-Token-Dispatch" = String, Header, description = "Optional token dispatch method. Values: cookie|json|both (case-insensitive; whitespace ignored). Defaults to both."),
            ("application_id" = String, Path, description = "Application ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<Option<SigninResponse>>),
            (status = 203, description = "Missing Authorization header"),
            (status = 400, description = "Invalid, expired, or revoked token", body = ApiResponse<ErrorResponse>),
            (status = 401, description = "Unauthorized"),
            (status = 403, description = "Forbidden - secret does not belong to this application"),
            (status = 404, description = "Application not found"),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "application_auth.refresh_legacy",
    skip(auth, token_mtd, revoked_jwt, keyboxes, applications, auditing, application_id),
    fields(
        user_id = field::Empty,
        application_id = field::Empty,
        old_jti = field::Empty,
        token_dispatch = field::Empty
    )
)]
pub async fn legacy_refresh_application_auth_token(
    auth: RequireAuth<Claim>,
    token_mtd: TokenDispatchMethod,
    _: RequireMatchedApplicationSecret,

    State(AppState {
        revoked_jwt,
        keyboxes,
        applications,
        auditing,
        ..
    }): State<AppState<'_>>,

    Path(application_id): Path<Sqid>,
) -> WithHeaderRestResult<Option<SigninResponse>> {
    let jti = auth.token.claims.jti;
    let user_id = auth.token.claims.sub;
    let application_id: Uuid = application_id.try_into().inspect_err(|e| {
        error!(
            %user_id,
            old_jti = %jti,
            error = %e,
            "failed to convert application_id"
        )
    })?;

    Span::current().tap(|it| {
        it.record("user_id", field::display(&user_id))
            .record("application_id", field::display(&application_id))
            .record("old_jti", field::display(&jti))
            .record("token_dispatch", field::debug(&token_mtd));
    });

    let ApplicationConfiguration { authentication } =
        applications.get_configuration(application_id).await?;

    info!(
        %user_id,
        %application_id,
        old_jti = %jti,
        "legacy token refresh requested"
    );

    if revoked_jwt.is_revoked(jti).await? {
        warn!(
            %user_id,
            %application_id,
            old_jti = %jti,
            "token refresh rejected: jwt already revoked"
        );
        return Err(Error::with_code(
            StatusCode::BAD_REQUEST,
            format!("jwt of jti={jti} has been revoked"),
        ));
    }

    revoked_jwt.set_revoked(jti).await.inspect_err(|e| {
        error!(
            %user_id,
            %application_id,
            old_jti = %jti,
            error = %e,
            "failed to revoke old jwt during refresh"
        )
    })?;

    info!(
        %user_id,
        %application_id,
        old_jti = %jti,
        "old jwt revoked successfully during refresh"
    );

    let EncodedJwt { jwt, claim } = keyboxes
        .sign_jwt::<Claim>(
            user_id,
            SignJwtOptions {
                application_id,
                iss: authentication.issuer,
                aud: authentication.audience,
            },
        )
        .await
        .inspect_err(|e| {
            error!(
                %user_id,
                %application_id,
                old_jti = %jti,
                error = %e,
                "failed to sign new jwt during refresh"
            )
        })?;

    info!(
        %user_id,
        %application_id,
        old_jti = %jti,
        "legacy token refresh successful"
    );

    auditing
        .write(AuditPayload::from(RefreshJwtPayload {
            application_id,
            subject_id: user_id,
            old_jti: jti,
            new_jti: claim.jti,
        }))
        .await;

    let cookie = Cookie::new("auth_token", jwt.clone());
    let resp = ApiResponseWithHeader::new(Some(SigninResponse { jwt }));

    let resp = match token_mtd {
        TokenDispatchMethod::Cookie => ApiResponseWithHeader::new(None).with_cookie(cookie)?,
        TokenDispatchMethod::Json => resp,
        TokenDispatchMethod::Both => resp.with_cookie(cookie)?,
    };

    Ok(resp)
}

mod spec_middlewares {
    use axum::{
        extract::FromRequestParts,
        http::{StatusCode, request::Parts},
    };
    use tracing::warn;
    use uuid::Uuid;

    use oceaniam_common::types::sqid::Sqid;

    use crate::{middlewares::application::RequireApplicationSecret, state::AppState};

    /// This middleware ensures that the provided secret matches the application_id in the path.
    ///
    /// If they don't match, the request will be rejected immediately.
    ///
    /// WARNING: This struct is intended for internal use within this crate only. It performs
    /// path-based application ID extraction which assumes specific URL patterns. Using it outside
    /// this crate may lead to unexpected behavior.
    #[derive(Debug, Clone)]
    #[allow(unused)]
    pub struct RequireMatchedApplicationSecret {
        pub secret: String,
        pub application_id: Uuid,
    }

    impl FromRequestParts<AppState<'_>> for RequireMatchedApplicationSecret {
        type Rejection = StatusCode;

        async fn from_request_parts(
            parts: &mut Parts,
            state: &AppState<'_>,
        ) -> Result<Self, Self::Rejection> {
            // First, validate the application secret
            let secret: RequireApplicationSecret =
                RequireApplicationSecret::from_request_parts(parts, state).await?;

            // Extract application_id from path
            let path = parts.uri.path();

            let path_segments: Vec<&str> = path.split('/').collect();

            // Find the application_id in path (format: /applications/{id}/...)
            let application_id = path_segments
                .iter()
                .position(|&s| s == "applications")
                .and_then(|idx| path_segments.get(idx + 1))
                .and_then(|id| id.parse::<Sqid>().ok())
                .and_then(|id| Uuid::try_from(id).ok())
                .ok_or_else(|| {
                    warn!(
                        "application authorization failed: cannot extract application_id from path"
                    );
                    StatusCode::BAD_REQUEST
                })?;

            // Verify the secret belongs to the requested application
            if !secret.is_matched(application_id) {
                warn!(
                    secret_application_ids = ?secret.of_applications,
                    requested_application_id = %application_id,
                    "application authorization failed: secret belongs to different application"
                );
                return Err(StatusCode::FORBIDDEN);
            }

            Ok(Self {
                secret: secret.secret,
                application_id,
            })
        }
    }
}
