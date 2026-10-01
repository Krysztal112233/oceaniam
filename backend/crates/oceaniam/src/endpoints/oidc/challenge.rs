//! Transaction-bound TOTP challenge verification for the OIDC authorization preview.
//!
//! This endpoint advances an `awaiting_challenge` authorization transaction to `authenticated`
//! with a valid one-time code. It shares the login endpoint's transport boundary (preview
//! gate, bounded lossy form parsing, no query parameters on POST), browser-binding cookie and
//! CSRF digests, ordered shared-lock registration revalidation, and uniform anti-enumeration
//! failure page. A live transaction in any other state, a wrong or replayed code, an expired
//! or exhausted challenge, and an inactive subject all return the exact login-failure page, so
//! the state-machine position and per-branch causes never leak and are never logged.

use std::convert::Infallible;

use axum::{
    extract::{DefaultBodyLimit, FromRequestParts, Path, State},
    http::{StatusCode, request::Parts},
    response::Response,
};
use axum_extra::extract::{Form, FormRejection, cookie::CookieJar};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use oceaniam_audit::types::{AuditPayload, OidcAuthenticatePayload};
use oceaniam_common::sqid::Sqid;
use oceaniam_database::{
    helper::{
        audits::AuditsHelper,
        challenges::ChallengesHelper,
        oidc_authorization_transactions::{
            AuthenticateChallengedAuthorizationTransactionInput,
            OidcAuthorizationTransactionsHelper,
        },
        subjects::SubjectsHelper,
    },
    model::{
        oidc_authorization_transactions,
        prelude::{Audits, Challenges, OidcAuthorizationTransactions, Subjects},
        sea_orm_active_enums::{AuditType, OidcAuthorizationTransactionStatus},
    },
};
use oceaniam_oidc::{
    AUTHORIZATION_FORM_MAX_BYTES, ParsedChallengeForm, authorization_browser_binding_digest,
    authorization_csrf_digest,
};
use sea_orm::{DatabaseTransaction, TransactionTrait};
use tracing::error;
use utoipa_axum::{
    router::{UtoipaMethodRouter, UtoipaMethodRouterExt},
    routes,
};
use uuid::Uuid;

use crate::state::AppState;

use super::authorize::{
    AuthorizationTarget, COOKIE_PREFIX, PreviewEnabled, form_rejection, local_response,
};
use super::login::{
    LoginFailure, SUCCESS_HTML, html_response, is_unavailable, login_failed_response,
    revalidate_login_registration, transaction_unavailable_response,
};

struct AuthorizationChallengePath {
    tenant_id: Uuid,
    transaction_id: Uuid,
}

impl FromRequestParts<AppState> for AuthorizationChallengePath {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Path((tenant_sqid, transaction_sqid)) =
            Path::<(String, String)>::from_request_parts(parts, state)
                .await
                .map_err(|_| local_response(StatusCode::BAD_REQUEST, "invalid challenge path"))?;
        let tenant_id = tenant_sqid
            .parse::<Sqid>()
            .ok()
            .and_then(|sqid| Uuid::try_from(sqid).ok())
            .ok_or_else(|| local_response(StatusCode::BAD_REQUEST, "invalid tenant identifier"))?;
        let transaction_id = transaction_sqid
            .parse::<Sqid>()
            .ok()
            .and_then(|sqid| Uuid::try_from(sqid).ok())
            .ok_or_else(|| {
                local_response(StatusCode::BAD_REQUEST, "invalid transaction identifier")
            })?;

        Ok(Self {
            tenant_id,
            transaction_id,
        })
    }
}

/// Default-disabled OIDC authorization challenge: verifies a TOTP code for an
/// `awaiting_challenge` transaction.
#[utoipa::path(
    post,
    path = "/oidc/{tenant_sqid}/authorize/{transaction_sqid}/challenge",
    tag = "Oidc",
    params(
        ("tenant_sqid" = String, Path, description = "Tenant Sqid"),
        ("transaction_sqid" = String, Path, description = "Authorization transaction Sqid"),
    ),
    request_body(
        content = String,
        content_type = "application/x-www-form-urlencoded",
        description = "Challenge parameters: the TOTP `code` and the transaction `csrf` secret; body-only, maximum 8 KiB"
    ),
    responses(
        (status = 200, description = "Authentication recorded; authorization continuation is not yet available", body = String, content_type = "text/html"),
        (status = 400, description = "Malformed challenge request or invalidated registration", body = String),
        (status = 401, description = "Indistinguishable challenge failure page", body = String, content_type = "text/html"),
        (status = 404, description = "Preview disabled, or the transaction binding is unavailable", body = String),
        (status = 405, description = "Only POST is supported", body = String),
        (status = 413, description = "Form body exceeds 8 KiB", body = String),
        (status = 414, description = "Request target exceeds 8 KiB", body = String),
        (status = 415, description = "Content type is not application/x-www-form-urlencoded", body = String),
        (status = 500, description = "Internal server error", body = String),
    ),
)]
#[tracing::instrument(level = "info", name = "oidc.authorization_challenge.post", skip_all)]
async fn post_authorization_challenge(
    _enabled: PreviewEnabled,
    _target: AuthorizationTarget,
    AuthorizationChallengePath {
        tenant_id,
        transaction_id,
    }: AuthorizationChallengePath,
    State(state): State<AppState>,
    cookies: CookieJar,
    form: Result<Form<ParsedChallengeForm>, FormRejection>,
) -> Response {
    let form = match form {
        Ok(Form(form)) => form,
        Err(rejection) => return form_rejection(rejection),
    };

    // The cookie name derives from the canonical transaction Sqid spelling used when the entry
    // endpoint set the cookie, regardless of the spelling supplied in the path.
    let cookie_name = format!("{COOKIE_PREFIX}{}", Sqid::from(transaction_id));
    let Some(browser_binding) = cookies
        .get(&cookie_name)
        .and_then(|cookie| URL_SAFE_NO_PAD.decode(cookie.value()).ok())
        .and_then(|bytes| <[u8; 32]>::try_from(bytes.as_slice()).ok())
    else {
        return transaction_unavailable_response();
    };
    let Some(csrf) = form
        .csrf()
        .and_then(|value| URL_SAFE_NO_PAD.decode(value).ok())
        .and_then(|bytes| <[u8; 32]>::try_from(bytes.as_slice()).ok())
    else {
        return transaction_unavailable_response();
    };
    let browser_binding_digest = authorization_browser_binding_digest(&browser_binding);
    let csrf_digest = authorization_csrf_digest(&csrf);

    let snapshot = match OidcAuthorizationTransactions::get_challenged_authorization_transaction(
        transaction_id,
        tenant_id,
        browser_binding_digest,
        csrf_digest,
        &state.database,
    )
    .await
    {
        Ok(snapshot) => snapshot,
        Err(error) if is_unavailable(&error) => return transaction_unavailable_response(),
        Err(error) => {
            error!(
                ?error,
                "failed to read OIDC authorization transaction for challenge"
            );
            return local_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error");
        }
    };

    // A live transaction in any other state returns the uniform credential-failure page so the
    // state-machine position never leaks.
    if snapshot.status != OidcAuthorizationTransactionStatus::AwaitingChallenge {
        return login_failed_response();
    }
    let Some(code) = form.code() else {
        return login_failed_response();
    };

    let transaction = match state.database.begin().await {
        Ok(transaction) => transaction,
        Err(error) => {
            error!(
                ?error,
                "failed to begin OIDC authorization-challenge transaction"
            );
            return local_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error");
        }
    };

    let outcome = challenge_in_transaction(
        &state,
        &transaction,
        tenant_id,
        &snapshot,
        browser_binding_digest,
        csrf_digest,
        code,
    )
    .await;

    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(failure) => {
            if transaction.rollback().await.is_err() {
                error!("failed to roll back OIDC authorization-challenge transaction");
            }
            return failure.into_response();
        }
    };

    if transaction.commit().await.is_err() {
        // Commit outcome can be ambiguous. Never retry and never report success.
        error!("failed to commit OIDC authorization-challenge transaction");
        return local_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error");
    }

    match outcome {
        ChallengeOutcome::Authenticated => html_response(StatusCode::OK, SUCCESS_HTML),
        ChallengeOutcome::AttemptRejected => login_failed_response(),
    }
}

/// Live revalidation, challenge locking, OTP verification, and the state transition in one
/// short transaction. A wrong or replayed code commits only the conditional attempt increment
/// and answers with the uniform failure page; a valid code consumes the challenge, transitions
/// the transaction, and audits the authentication atomically.
async fn challenge_in_transaction(
    state: &AppState,
    transaction: &DatabaseTransaction,
    tenant_id: Uuid,
    snapshot: &oidc_authorization_transactions::Model,
    browser_binding_digest: [u8; 32],
    csrf_digest: [u8; 32],
    code: &str,
) -> Result<ChallengeOutcome, LoginFailure> {
    revalidate_login_registration(tenant_id, snapshot, transaction).await?;

    // `challenge_authorization_transaction` always writes the binding atomically with the
    // `awaiting_challenge` status, so a missing column is an internal inconsistency.
    let challenge_id = snapshot.challenge_id.ok_or(LoginFailure::Internal)?;
    let challenge =
        Challenges::lock_challenge_for_attempt(challenge_id, snapshot.application_id, transaction)
            .await
            .map_err(|error| {
                if is_unavailable(&error) {
                    LoginFailure::LoginFailed
                } else {
                    LoginFailure::Internal
                }
            })?;

    // OTP verification (vault lookup and anti-replay consume) is deliberately outside the
    // database transaction's rollback scope, matching the committed signin challenge step.
    let verified = state
        .credentials
        .verify_totp(challenge.subject_id, code)
        .await
        .map_err(|_| LoginFailure::Internal)?;
    if !verified {
        Challenges::increment_challenge_attempt(challenge.id, transaction)
            .await
            .map_err(|error| {
                if is_unavailable(&error) {
                    LoginFailure::LoginFailed
                } else {
                    LoginFailure::Internal
                }
            })?;
        return Ok(ChallengeOutcome::AttemptRejected);
    }

    Challenges::consume_pending_challenge(challenge.id, transaction)
        .await
        .map_err(|error| {
            if is_unavailable(&error) {
                LoginFailure::LoginFailed
            } else {
                LoginFailure::Internal
            }
        })?;

    OidcAuthorizationTransactions::authenticate_challenged_authorization_transaction(
        AuthenticateChallengedAuthorizationTransactionInput {
            id: snapshot.id,
            browser_binding_digest,
            csrf_digest,
            expected_revision: snapshot.revision,
            challenge_id,
            subject_id: challenge.subject_id,
        },
        transaction,
    )
    .await
    .map_err(|error| {
        if is_unavailable(&error) {
            LoginFailure::LoginFailed
        } else {
            LoginFailure::Internal
        }
    })?;

    // Lazy expiration rejection on the completing leg, checked only after the code matches (see
    // the committed signin challenge step), so the generic failure cannot probe expiration.
    Subjects::ensure_subject_active(challenge.subject_id, transaction)
        .await
        .map_err(|_| LoginFailure::LoginFailed)?;

    let payload = AuditPayload::from(OidcAuthenticatePayload {
        application_id: snapshot.application_id,
        transaction_id: snapshot.id,
        subject_id: challenge.subject_id,
    })
    .into_json()
    .map_err(|_| LoginFailure::Internal)?;
    Audits::insert_audit_event(
        Uuid::now_v7(),
        AuditType::OidcAuthenticate,
        payload,
        transaction,
    )
    .await
    .map_err(|_| LoginFailure::Internal)?;

    Ok(ChallengeOutcome::Authenticated)
}

enum ChallengeOutcome {
    Authenticated,
    AttemptRejected,
}

pub(super) fn routes() -> UtoipaMethodRouter<AppState, Infallible> {
    routes!(post_authorization_challenge)
        .map(|methods| methods.route_layer(DefaultBodyLimit::max(AUTHORIZATION_FORM_MAX_BYTES)))
}
