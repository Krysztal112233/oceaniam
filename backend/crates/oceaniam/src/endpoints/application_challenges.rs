//! Application challenge-related API endpoints

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use axum_extra::extract::cookie::Cookie;
use oceaniam_api::{
    ApiResponse, ApiResponseWithHeader, ErrorResponse, RestResult, WithHeaderRestResult,
};
use oceaniam_audit::types::{AuditPayload, SignJwtPayload};
use oceaniam_auth::jwt::Claim;
use oceaniam_common::error::Error;
use oceaniam_database::{
    config::application::ApplicationConfiguration, helper::challenges::ChallengesHelper,
    model::prelude::Challenges,
};
use oceaniam_vo::{
    applications::ApplicationChallengeVO,
    auth::{SigninResponseOrChallenge, SignupResponse},
};
use serde_json::Value;
use tap::Tap;
use tracing::{Span, error, field, info};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    endpoints::applications::{TenantApplicationPath, get_tenant_application},
    middlewares::{
        application::RequireAdminJwtOrMatchedApplicationSecret, auth::TokenDispatchMethod,
    },
    state::AppState,
    state::keybox::{EncodedJwt, SignJwtOptions},
};

pub fn endpoint<'a: 'static>(router: OpenApiRouter<AppState<'a>>) -> OpenApiRouter<AppState<'a>> {
    router
        .routes(routes!(get_application_challenge))
        .routes(routes!(create_application_challenge_attempt))
}

#[derive(Debug, serde::Deserialize, ToSchema)]
pub(crate) struct ApplicationChallengePath {
    #[serde(flatten)]
    pub application: TenantApplicationPath,
    pub challenge_id: Uuid,
}

/// Get application challenge detail
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
    skip(database, path),
    fields(application_id = field::Empty, challenge_id = field::Empty)
)]
pub async fn get_application_challenge(
    _: RequireAdminJwtOrMatchedApplicationSecret,
    State(AppState { database, .. }): State<AppState<'_>>,
    Path(path): Path<ApplicationChallengePath>,
) -> RestResult<ApplicationChallengeVO> {
    let application = get_tenant_application(path.application, &database).await?;
    let application_id = application.id;
    let challenge_id = path.challenge_id;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&application.tenant_id))
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

    Ok(ApiResponse::new(challenge.into()))
}

/// Create application challenge attempt
#[utoipa::path(
        post,
        path = "/tenants/{tenant_id}/applications/{application_id}/challenges/{challenge_id}",
        tag = "ApplicationChallenges",
        params(
            ("Authorization" = String, Header, description = "Bearer token for backend administrator"),
            ("X-OceanIAM-Application-Secret" = String, Header, description = "Application secret"),
            ("X-OceanIAM-Token-Dispatch" = String, Header, description = "Optional token dispatch method. Values: cookie|json|both (case-insensitive; whitespace ignored). Defaults to both."),
            ("tenant_id" = String, Path, description = "Tenant ID"),
            ("application_id" = String, Path, description = "Application ID"),
            ("challenge_id" = String, Path, description = "Challenge ID"),
        ),
        request_body = Value,
        responses(
            (status = 200, body = ApiResponse<Option<SigninResponseOrChallenge>>),
            (status = 203, description = "Missing Authorization header and application secret header"),
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
    skip(challenges, keyboxes, applications, auditing, token_mtd, path, payload, database),
    fields(application_id = field::Empty, challenge_id = field::Empty, user_id = field::Empty, token_dispatch = field::Empty)
)]
pub async fn create_application_challenge_attempt(
    _: RequireAdminJwtOrMatchedApplicationSecret,
    token_mtd: TokenDispatchMethod,
    State(AppState {
        challenges,
        database,
        keyboxes,
        applications,
        auditing,
        ..
    }): State<AppState<'_>>,
    Path(path): Path<ApplicationChallengePath>,
    Json(payload): Json<Value>,
) -> WithHeaderRestResult<SigninResponseOrChallenge> {
    let application = get_tenant_application(path.application, &database).await?;
    let application_id = application.id;
    let challenge_id = path.challenge_id;
    let challenge = challenges
        .get_challenge(application_id, challenge_id)
        .await
        .inspect_err(
            |e| error!(%application_id, %challenge_id, error = %e, "failed to get challenge"),
        )?;
    let user_id = challenge.subject_id;
    Span::current().tap(|it| {
        it.record("tenant_id", field::display(&application.tenant_id))
            .record("application_id", field::display(&application_id))
            .record("challenge_id", field::display(&challenge_id))
            .record("user_id", field::display(&user_id))
            .record("token_dispatch", field::debug(&token_mtd));
    });

    challenges
        .verify_challenge(application_id, challenge_id, payload)
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
                application_id,
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

    let cookie = Cookie::new("auth_token", jwt.clone());
    let resp =
        ApiResponseWithHeader::new(SigninResponseOrChallenge::Signup(SignupResponse { jwt }));

    let resp = match token_mtd {
        TokenDispatchMethod::Cookie => ApiResponseWithHeader::empty().with_cookie(cookie)?,
        TokenDispatchMethod::Json => resp,
        TokenDispatchMethod::Both => resp.with_cookie(cookie)?,
    };

    Ok(resp)
}
