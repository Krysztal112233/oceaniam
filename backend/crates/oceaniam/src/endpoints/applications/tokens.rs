use axum::{Json, extract::State, http::StatusCode};
use oceaniam_api::ApiResponse;
use oceaniam_audit::types::{AuditPayload, SignJwtPayload};
use oceaniam_auth::jwt::Claim;
use oceaniam_common::consts;
use oceaniam_database::config::application::ApplicationConfiguration;
use oceaniam_database::helper::challenges::CreateChallengeOpts;
use oceaniam_database::model::sea_orm_active_enums::{ChallengeFactorType, ChallengePurposeType};
use oceaniam_vo::auth::{AuthVO, SigninChallenge, SigninResponseOrChallenge};
use tap::Tap;
use tracing::{Span, error, field, info};
use utoipa_axum::{router::OpenApiRouter, routes};

use super::ResolvedApplication;
use crate::{
    conversion::sqid::uuid_to_sqid,
    error::{AppResult, Error},
    middlewares::{application::MatchedApplicationSecretGuard, auth::TokenDispatchMethodGuard},
    state::{
        AppState,
        keybox::{EncodedJwt, SignJwtOptions},
    },
    util::token_response::dispatch_signin_response,
};

mod refresh;
mod signout;

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router
        .routes(routes!(create_application_token))
        .routes(routes!(signout::delete_application_token))
        .routes(routes!(refresh::refresh_application_token))
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
            (status = 401, description = "Invalid credentials (all failures return 401 to prevent enumeration)"),
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
        otel.kind = "internal",
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
        cookie,
        ..
    }): State<AppState>,
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
            StatusCode::UNAUTHORIZED,
            consts::USER_LOGIN_FAILED_MSG,
        ));
    }

    if credentials.has_totp(user.id).await? {
        let challenges = applications.challenges(application_id).await?;
        let challenge = challenges
            .create_challenge(
                user.id,
                CreateChallengeOpts {
                    factor_type: ChallengeFactorType::Totp,
                    challenge_purpose_type: ChallengePurposeType::Signin,
                    ..Default::default()
                },
            )
            .await?;

        info!(
            %application_id,
            user_id = %user.id,
            challenge_id = %challenge.id,
            "mfa challenge created during signin"
        );

        return Ok(ApiResponse::new(SigninResponseOrChallenge::Challenge(
            SigninChallenge {
                challenge_id: uuid_to_sqid(challenge.id),
                factor_type: "totp".to_string(),
                expires_at: challenge.expires_at,
            },
        )));
    }

    let EncodedJwt { jwt, claim } = keyboxes
        .sign_jwt::<Claim>(
            user.id,
            SignJwtOptions {
                tenant_id: app.tenant_id(),
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

    dispatch_signin_response(jwt, &token_mtd, cookie.secure)
}
