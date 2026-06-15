use axum::extract::State;
use oceaniam_api::{ApiResponse, ErrorResponse};
use oceaniam_audit::types::{AuditPayload, RevokeJwtPayload};
use oceaniam_vo::auth::SignoutResponse;
use tap::Tap;
use tracing::{Span, error, field, info};

use super::super::ResolvedApplication;
use crate::{
    error::Error,
    middlewares::{application::MatchedApplicationSecretGuard, auth::ApplicationAuthGuard},
    state::AppState,
};

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
    }): State<AppState>,
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
