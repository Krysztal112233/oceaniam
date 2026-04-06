//! Application token-related API endpoints

use crate::{
    endpoints::applications::{TenantApplicationPath, get_tenant_application},
    middlewares::{
        application::RequireMatchedApplicationSecret,
        auth::{RequireAuth, TokenDispatchMethod},
    },
    state::{
        AppState,
        keybox::{EncodedJwt, SignJwtOptions},
    },
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use axum_extra::extract::cookie::Cookie;
use oceaniam_audit::types::{AuditPayload, RefreshJwtPayload, RevokeJwtPayload, SignJwtPayload};
use oceaniam_common::{
    ApiResponse, ApiResponseWithHeader, ErrorResponse, WithHeaderRestResult, consts, error::Error,
    jwt::Claim,
};
use oceaniam_credential::credential::Password;
use oceaniam_database::helper::applications::ApplicationConfiguration;
use oceaniam_vo::auth::{AuthVO, SigninResponse, SignoutResponse};
use tap::Tap;
use tracing::{Span, error, field, info, warn};
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState<'a>>) -> OpenApiRouter<AppState<'a>> {
    router
        .routes(routes!(create_application_token))
        .routes(routes!(delete_application_token))
        .routes(routes!(refresh_application_token))
}

/// Create application token
#[utoipa::path(
        post,
        path = "/tenants/{tenant_id}/applications/{application_id}/tokens",
        tag = "ApplicationTokens",
        params(
            ("X-OceanIAM-Application-Secret" = String, Header, description = "Application secret"),
            ("X-OceanIAM-Token-Dispatch" = String, Header, description = "Optional token dispatch method. Values: cookie|json|both (case-insensitive; whitespace ignored). Defaults to both."),
            ("tenant_id" = String, Path, description = "Tenant ID"),
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
    name = "tenant_application_tokens.create",
    skip(token_mtd, applications, credentials, keyboxes, auditing, path, auth, database),
    fields(
        tenant_id = field::Empty,
        application_id = field::Empty,
        user_id = field::Empty,
        token_dispatch = field::Empty
    )
)]
pub async fn create_application_token(
    _: RequireMatchedApplicationSecret,
    token_mtd: TokenDispatchMethod,
    State(AppState {
        database,
        applications,
        credentials,
        keyboxes,
        auditing,
        ..
    }): State<AppState<'_>>,
    Path(path): Path<TenantApplicationPath>,
    Json(auth): Json<AuthVO>,
) -> WithHeaderRestResult<Option<SigninResponse>> {
    let application = get_tenant_application(path, &database).await?;
    let application_id = application.id;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&application.tenant_id))
            .record("application_id", field::display(&application_id))
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
                "failed to sign jwt during signin"
            )
        })?;

    info!(%application_id, user_id = %user.id, "signin successful");

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

/// Delete application token
#[utoipa::path(
        delete,
        path = "/tenants/{tenant_id}/applications/{application_id}/tokens",
        tag = "ApplicationTokens",
        params(
            ("Authorization" = String, Header, description = "Bearer token"),
            ("X-OceanIAM-Application-Secret" = String, Header, description = "Application secret"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
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
    name = "tenant_application_tokens.delete",
    skip(auth, revoked_jwt, auditing, path, database),
    fields(user_id = field::Empty, tenant_id = field::Empty, application_id = field::Empty, jti = field::Empty)
)]
pub async fn delete_application_token(
    _: RequireMatchedApplicationSecret,
    auth: RequireAuth<Claim>,
    State(AppState {
        database,
        revoked_jwt,
        auditing,
        ..
    }): State<AppState<'_>>,
    Path(path): Path<TenantApplicationPath>,
) -> Result<ApiResponse<SignoutResponse>, Error> {
    let jti = auth.token.claims.jti;
    let user_id = auth.token.claims.sub;
    let application = get_tenant_application(path, &database).await?;
    let app_id = application.id;
    Span::current().tap(|it| {
        it.record("user_id", field::display(&user_id))
            .record("tenant_id", field::display(&application.tenant_id))
            .record("application_id", field::display(&app_id))
            .record("jti", field::display(&jti));
    });

    info!(%user_id, application_id = %app_id, %jti, "signout requested");

    revoked_jwt.set_revoked(jti).await.inspect_err(|e| {
        error!(
            %user_id,
            application_id = %app_id,
            %jti,
            error = %e,
            "failed to revoke jwt during signout"
        )
    })?;

    auditing
        .write(AuditPayload::from(RevokeJwtPayload {
            subject_id: user_id,
            jti,
            application_id: Some(app_id),
        }))
        .await;

    Ok(ApiResponse::new(SignoutResponse::default()))
}

/// Refresh application token
#[utoipa::path(
        post,
        path = "/tenants/{tenant_id}/applications/{application_id}/tokens/refresh",
        tag = "ApplicationTokens",
        params(
            ("Authorization" = String, Header, description = "Bearer token to refresh"),
            ("X-OceanIAM-Application-Secret" = String, Header, description = "Application secret"),
            ("X-OceanIAM-Token-Dispatch" = String, Header, description = "Optional token dispatch method. Values: cookie|json|both (case-insensitive; whitespace ignored). Defaults to both."),
            ("tenant_id" = String, Path, description = "Tenant ID"),
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
    name = "tenant_application_tokens.refresh",
    skip(auth, token_mtd, revoked_jwt, keyboxes, applications, auditing, path, database),
    fields(
        user_id = field::Empty,
        tenant_id = field::Empty,
        application_id = field::Empty,
        old_jti = field::Empty,
        token_dispatch = field::Empty
    )
)]
pub async fn refresh_application_token(
    auth: RequireAuth<Claim>,
    token_mtd: TokenDispatchMethod,
    _: RequireMatchedApplicationSecret,
    State(AppState {
        database,
        revoked_jwt,
        keyboxes,
        applications,
        auditing,
        ..
    }): State<AppState<'_>>,
    Path(path): Path<TenantApplicationPath>,
) -> WithHeaderRestResult<Option<SigninResponse>> {
    let jti = auth.token.claims.jti;
    let user_id = auth.token.claims.sub;
    let application = get_tenant_application(path, &database).await?;
    let application_id = application.id;

    Span::current().tap(|it| {
        it.record("user_id", field::display(&user_id))
            .record("tenant_id", field::display(&application.tenant_id))
            .record("application_id", field::display(&application_id))
            .record("old_jti", field::display(&jti))
            .record("token_dispatch", field::debug(&token_mtd));
    });

    let ApplicationConfiguration { authentication, .. } =
        applications.get_configuration(application_id).await?;

    info!(%user_id, %application_id, old_jti = %jti, "token refresh requested");

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
