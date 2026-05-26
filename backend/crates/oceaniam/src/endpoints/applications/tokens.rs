use axum::{Json, extract::State, http::StatusCode};
use oceaniam_api::{ApiResponse, ErrorResponse};
use oceaniam_audit::types::{AuditPayload, RefreshJwtPayload, RevokeJwtPayload, SignJwtPayload};
use oceaniam_auth::jwt::Claim;
use oceaniam_common::consts;
use oceaniam_database::config::application::ApplicationConfiguration;
use oceaniam_vo::auth::{AuthVO, SigninResponseOrChallenge, SignoutResponse, SignupResponse};
use tap::Tap;
use tracing::{Span, error, field, info, warn};
use utoipa_axum::{router::OpenApiRouter, routes};

use super::ResolvedApplication;
use crate::{
    error::{AppResult, Error},
    middlewares::{
        application::MatchedApplicationSecretGuard,
        auth::{ApplicationAuthGuard, TokenDispatchMethodGuard},
    },
    state::{
        AppState,
        keybox::{EncodedJwt, SignJwtOptions},
    },
    util::cookie::build_auth_cookie,
};

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
            ("X-OceanIAM-Token-Dispatch" = Option<String>, Header, description = "Optional token dispatch method. Values: cookie|json|both (case-insensitive; whitespace ignored). Defaults to both."),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("application_id" = String, Path, description = "Application ID"),
        ),
        request_body = AuthVO,
        responses(
            (status = 200, body = ApiResponse<SigninResponseOrChallenge>),
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
    skip(token_mtd, applications, credentials, keyboxes, auditing, auth),
    fields(
        tenant_id = field::Empty,
        application_id = field::Empty,
        user_id = field::Empty,
        token_dispatch = field::Empty
    )
)]
pub async fn create_application_token(
    _: MatchedApplicationSecretGuard,
    token_mtd: TokenDispatchMethodGuard,
    State(AppState {
        applications,
        credentials,
        keyboxes,
        auditing,
        config,
        ..
    }): State<AppState<'_>>,
    app: ResolvedApplication,
    Json(auth): Json<AuthVO>,
) -> AppResult<SigninResponseOrChallenge> {
    let application_id = app.id();
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id))
            .record("token_dispatch", field::debug(&token_mtd));
    });

    let ApplicationConfiguration {
        auth: authentication,
        ..
    } = applications
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

    let verify_result = match auth {
        AuthVO::Email { password, .. } | AuthVO::Phone { password, .. } => credentials
            .verify_password(user.id, &password)
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
                iss: authentication.token.issuer,
                aud: authentication.token.audience,
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

    let cookie = build_auth_cookie(&jwt, config.cookie.secure);
    let resp = ApiResponse::new(SigninResponseOrChallenge::Signup(SignupResponse { jwt }));

    let resp = match token_mtd {
        TokenDispatchMethodGuard::Cookie => ApiResponse::empty().with_cookie(cookie)?,
        TokenDispatchMethodGuard::Json => resp,
        TokenDispatchMethodGuard::Both => resp.with_cookie(cookie)?,
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
    skip(auth, revoked_jwt, auditing),
    fields(user_id = field::Empty, tenant_id = field::Empty, application_id = field::Empty, jti = field::Empty)
)]
pub async fn delete_application_token(
    _: MatchedApplicationSecretGuard,
    auth: ApplicationAuthGuard,
    State(AppState {
        revoked_jwt,
        auditing,
        ..
    }): State<AppState<'_>>,
    app: ResolvedApplication,
) -> Result<ApiResponse<SignoutResponse>, Error> {
    let jti = auth.token.claims.jti;
    let user_id = auth.token.claims.sub;
    let app_id = app.id();
    Span::current().tap(|it| {
        it.record("user_id", field::display(&user_id))
            .record("tenant_id", field::display(&app.tenant_id()))
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
            ("X-OceanIAM-Token-Dispatch" = Option<String>, Header, description = "Optional token dispatch method. Values: cookie|json|both (case-insensitive; whitespace ignored). Defaults to both."),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("application_id" = String, Path, description = "Application ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<SigninResponseOrChallenge>),
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
    skip(auth, token_mtd, revoked_jwt, keyboxes, applications, auditing),
    fields(
        user_id = field::Empty,
        tenant_id = field::Empty,
        application_id = field::Empty,
        old_jti = field::Empty,
        token_dispatch = field::Empty
    )
)]
pub async fn refresh_application_token(
    auth: ApplicationAuthGuard,
    token_mtd: TokenDispatchMethodGuard,
    _: MatchedApplicationSecretGuard,
    State(AppState {
        revoked_jwt,
        keyboxes,
        applications,
        auditing,
        config,
        ..
    }): State<AppState<'_>>,
    app: ResolvedApplication,
) -> AppResult<SigninResponseOrChallenge> {
    let jti = auth.token.claims.jti;
    let user_id = auth.token.claims.sub;
    let application_id = app.id();

    Span::current().tap(|it| {
        it.record("user_id", field::display(&user_id))
            .record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id))
            .record("old_jti", field::display(&jti))
            .record("token_dispatch", field::debug(&token_mtd));
    });

    let ApplicationConfiguration {
        auth: authentication,
        ..
    } = applications.get_configuration(application_id).await?;

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
                iss: authentication.token.issuer,
                aud: authentication.token.audience,
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

    let cookie = build_auth_cookie(&jwt, config.cookie.secure);
    let resp = ApiResponse::new(SigninResponseOrChallenge::Signup(SignupResponse { jwt }));

    let resp = match token_mtd {
        TokenDispatchMethodGuard::Cookie => ApiResponse::empty().with_cookie(cookie)?,
        TokenDispatchMethodGuard::Json => resp,
        TokenDispatchMethodGuard::Both => resp.with_cookie(cookie)?,
    };

    Ok(resp)
}
