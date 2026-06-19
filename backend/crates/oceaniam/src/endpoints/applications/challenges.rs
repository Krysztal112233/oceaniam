use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use oceaniam_api::{ApiResponse, ErrorResponse};
use oceaniam_audit::types::{AuditPayload, SignJwtPayload};
use oceaniam_auth::jwt::Claim;
use oceaniam_common::sqid::Sqid;
use oceaniam_database::{
    config::application::ApplicationConfiguration,
    helper::challenges::{ChallengesHelper, CreateChallengeOpts},
    model::prelude::Challenges,
    model::sea_orm_active_enums::ChallengeFactorType,
};
use oceaniam_vo::{applications::ApplicationChallengeVO, auth::SigninResponseOrChallenge};
use serde::Deserialize;
use serde_json::Value;
use tap::Tap;
use tracing::{Span, error, field, info};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::ResolvedApplication;
use crate::{
    error::{AppResult, Error},
    middlewares::{application::AdminJwtOrApplicationSecretGuard, auth::TokenDispatchMethodGuard},
    state::AppState,
    state::keybox::{EncodedJwt, SignJwtOptions},
    util::token_response::dispatch_signin_response,
};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateChallengeRequest {
    pub subject_id: Uuid,
    pub factor_type: Option<String>,
    pub payload: Option<Value>,
}

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState>) -> OpenApiRouter<AppState> {
    router
        .routes(routes!(get_application_challenge))
        .routes(routes!(create_application_challenge))
        .routes(routes!(create_application_challenge_attempt))
}

/// Retrieve a pending challenge's metadata by its ID.
///
/// Returns the challenge details (factor type, purpose, status, expiry) for a challenge that
/// belongs to the specified application. The caller must authenticate either as a backend
/// administrator (Bearer JWT) or with the application's own secret
/// (`X-OceanIAM-Application-Secret`).
///
/// Once a challenge is consumed, expired, or no longer in `Pending` status, this endpoint returns
/// `404 Not Found`.
#[utoipa::path(
        get,
        path = "/tenants/{tenant_id}/applications/{application_id}/challenges/{challenge_id}",
        tag = "ApplicationChallenges",
        params(
            ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
            ("X-OceanIAM-Application-Secret" = String, Header, description = "Application secret"),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("application_id" = String, Path, description = "Application ID"),
            ("challenge_id" = String, Path, description = "Challenge ID"),
        ),
        responses(
            (status = 200, body = ApiResponse<ApplicationChallengeVO>),
            (status = 203, description = "Missing Authorization header and application secret header"),
            (status = 400, description = "Invalid ids", body = ApiResponse<ErrorResponse>),
            (status = 401, description = "Unauthorized"),
            (status = 403, description = "Forbidden - secret does not belong to this application"),
            (status = 404, description = "Application or challenge not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "application_challenges.get",
    fields(application_id = field::Empty, challenge_id = field::Empty)
)]
pub async fn get_application_challenge(
    _: AdminJwtOrApplicationSecretGuard,
    State(AppState { database, .. }): State<AppState>,
    app: ResolvedApplication,
    Path((_tid, _aid, challenge_id)): Path<(Sqid, Sqid, Sqid)>,
) -> AppResult<ApplicationChallengeVO> {
    let challenge_id: Uuid = challenge_id.try_into()?;
    let application_id = app.id();
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id))
            .record("challenge_id", field::display(&challenge_id));
    });

    let challenge = Challenges::get_challenge(challenge_id, &database)
        .await
        .inspect_err(
            |e| error!(%application_id, %challenge_id, error = %e, "failed to get challenge"),
        )?;

    if challenge.application_id != application_id {
        return Err(Error::with_code(
            StatusCode::NOT_FOUND,
            format!(
                "challenge_id={} not found under application_id={}",
                challenge.id, application_id
            ),
        ));
    }

    Ok(ApiResponse::new(
        crate::conversion::challenges::challenge_model_to_vo(challenge),
    ))
}

/// Create a challenge for MFA verification
///
/// Creates a new challenge for the specified subject. The client can then present the challenge to
/// the user and submit the verification via `POST .../challenges/{challenge_id}`.
///
/// Authentication is required via either a backend administrator Bearer JWT or the application's
/// own `X-OceanIAM-Application-Secret`.
#[utoipa::path(
    post,
    path = "/tenants/{tenant_id}/applications/{application_id}/challenges",
    tag = "ApplicationChallenges",
    params(
        ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
        ("X-OceanIAM-Application-Secret" = String, Header, description = "Application secret"),
        ("tenant_id" = String, Path, description = "Tenant ID"),
        ("application_id" = String, Path, description = "Application ID"),
    ),
    request_body = CreateChallengeRequest,
    responses(
        (status = 200, body = ApiResponse<ApplicationChallengeVO>),
        (status = 400, description = "Invalid ids or request body"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - secret does not belong to this application"),
        (status = 404, description = "Application not found"),
        (status = 500, description = "Internal server error"),
    ),
)]
#[tracing::instrument(
    level = "info",
    name = "application_challenges.create",
    skip(_auth, applications, body),
    fields(application_id = field::Empty, subject_id = field::Empty, challenge_id = field::Empty)
)]
pub async fn create_application_challenge(
    _auth: AdminJwtOrApplicationSecretGuard,
    State(AppState { applications, .. }): State<AppState>,
    app: ResolvedApplication,
    Json(body): Json<CreateChallengeRequest>,
) -> AppResult<ApplicationChallengeVO> {
    let application_id = app.id();
    Span::current().tap(|it| {
        it.record("application_id", field::display(&application_id))
            .record("subject_id", field::display(&body.subject_id));
    });

    let challenges = applications.challenges(application_id).await?;
    let factor_type = body.factor_type.as_deref().unwrap_or("totp");
    let factor = match factor_type {
        "email_totp" => ChallengeFactorType::EmailTotp,
        "totp" => ChallengeFactorType::Totp,
        _ => {
            return Err(Error::with_code(
                StatusCode::BAD_REQUEST,
                format!("unsupported factor type: {factor_type}"),
            ));
        }
    };

    let challenge = challenges
        .create_challenge(
            body.subject_id,
            CreateChallengeOpts {
                factor_type: factor,
                payload: body.payload,
                ..Default::default()
            },
        )
        .await?;

    Span::current().tap(|it| {
        it.record("challenge_id", field::display(&challenge.id));
    });

    Ok(ApiResponse::new(
        crate::conversion::challenges::challenge_model_to_vo(challenge),
    ))
}

/// Submit a verification payload for a pending challenge and, on success, receive a signed JWT.
///
/// This is the second step of the application MFA challenge flow (after a challenge has been
/// created by the application during sign-in). The caller provides the MFA verification payload
/// (e.g. a TOTP code) as a JSON body. No authentication is required because security is provided
/// by the challenge itself (short TTL, max attempts).
///
/// On successful verification the challenge is marked as `Consumed` and a signed application JWT is
/// issued for the subject that owns the challenge. The token can be dispatched as a JSON body, as
/// an `auth_token` cookie, or both, controlled by the `X-OceanIAM-Token-Dispatch` header.
#[utoipa::path(
        post,
        path = "/tenants/{tenant_id}/applications/{application_id}/challenges/{challenge_id}",
        tag = "ApplicationChallenges",
        params(
            ("X-OceanIAM-Token-Dispatch" = Option<String>, Header, description = "Optional token dispatch method. Values: cookie|json|both (case-insensitive; whitespace ignored). Defaults to both."),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("application_id" = String, Path, description = "Application ID"),
            ("challenge_id" = String, Path, description = "Challenge ID"),
        ),
        request_body = Value,
        responses(
            (status = 400, description = "Invalid ids or request body", body = ApiResponse<ErrorResponse>),
            (status = 401, description = "Unauthorized"),
            (status = 403, description = "Forbidden - secret does not belong to this application"),
            (status = 404, description = "Application or challenge not found", body = ApiResponse<ErrorResponse>),
            (status = 500, description = "Internal server error", body = ApiResponse<ErrorResponse>),
        ),
    )]
#[tracing::instrument(
    level = "info",
    name = "application_challenges.create_attempt",
    skip(keyboxes, applications, auditing, token_mtd, payload),
    fields(application_id = field::Empty, challenge_id = field::Empty, user_id = field::Empty, token_dispatch = field::Empty)
)]
pub async fn create_application_challenge_attempt(
    token_mtd: TokenDispatchMethodGuard,
    State(AppState {
        keyboxes,
        applications,
        auditing,
        config,
        ..
    }): State<AppState>,
    app: ResolvedApplication,
    Path((_tid, _aid, challenge_id)): Path<(Sqid, Sqid, Sqid)>,
    Json(payload): Json<Value>,
) -> AppResult<SigninResponseOrChallenge> {
    let challenge_id: Uuid = challenge_id.try_into()?;
    let application_id = app.id();
    let challenges = applications.challenges(application_id).await.inspect_err(
        |e| error!(%application_id, %challenge_id, error = %e, "failed to get challenges manager"),
    )?;
    let challenge = challenges.get_challenge(challenge_id).await.inspect_err(
        |e| error!(%application_id, %challenge_id, error = %e, "failed to get challenge"),
    )?;
    let user_id = challenge.subject_id;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&app.tenant_id()))
            .record("application_id", field::display(&application_id))
            .record("challenge_id", field::display(&challenge_id))
            .record("user_id", field::display(&user_id))
            .record("token_dispatch", field::debug(&token_mtd));
    });

    challenges
        .verify_challenge(challenge_id, payload)
        .await
        .inspect_err(
            |e| error!(%application_id, %challenge_id, error = %e, "failed to verify challenge"),
        )?;

    let ApplicationConfiguration {
        auth: authentication,
        ..
    } = applications.get_configuration(application_id).await?;

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
                %application_id,
                %user_id,
                %challenge_id,
                error = %e,
                "failed to sign jwt after challenge verification"
            )
        })?;

    info!(%application_id, %user_id, %challenge_id, "challenge verified and signin completed");

    auditing
        .write(AuditPayload::from(SignJwtPayload {
            application_id,
            subject_id: user_id,
            jti: claim.jti,
        }))
        .await;

    dispatch_signin_response(jwt, &token_mtd, config.cookie.secure)
}
