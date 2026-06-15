use axum::{extract::State, http::StatusCode};
use oceaniam_api::{ApiResponse, ErrorResponse};
use oceaniam_audit::types::{AuditPayload, RefreshJwtPayload};
use oceaniam_auth::jwt::Claim;
use oceaniam_database::config::application::ApplicationConfiguration;
use oceaniam_vo::auth::SigninResponseOrChallenge;
use tap::Tap;
use tracing::{Span, error, field, info, warn};

use super::super::ResolvedApplication;
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
    util::token_response::dispatch_signin_response,
};

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
    }): State<AppState>,
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
                tenant_id: app.tenant_id(),
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

    dispatch_signin_response(jwt, &token_mtd, config.cookie.secure)
}
