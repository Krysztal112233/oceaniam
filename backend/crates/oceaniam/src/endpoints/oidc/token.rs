//! OIDC token endpoint: RFC 6749 §3.2 authorization-code exchange.
//!
//! Redeems an authorization code exactly once and issues a tenant-keybox-signed JWT access
//! token and an OIDC ID token, each with a one-hour lifetime. The request's `client_id`,
//! `redirect_uri`, and PKCE `code_verifier` are validated only after the code is consumed, so
//! a failed exchange still burns the code (fail-closed) and binding mismatches cannot be
//! probed online; internal signing or storage failures roll back and leave the code redeemable
//! for a retry. No refresh token is issued, and the endpoint stays outside the CORS layer with
//! the rest of the authorization-endpoint family. The plaintext code, verifier, and issued
//! tokens are never logged.

use std::convert::Infallible;

use axum::{
    Json,
    extract::{DefaultBodyLimit, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::extract::{Form, FormRejection};
use oceaniam_audit::types::{AuditPayload, IssueTokenPayload};
use oceaniam_auth::jwt::{Claim, IdTokenClaim};
use oceaniam_common::sqid::Sqid;
use oceaniam_database::{
    helper::{
        audits::AuditsHelper, oidc_authorization_codes::OidcAuthorizationCodesHelper,
        oidc_authorization_transactions::OidcAuthorizationTransactionsHelper,
        oidc_clients::OidcClientsHelper, users::UserHelper,
    },
    model::{
        oidc_authorization_codes,
        prelude::{
            Audits, OidcAuthorizationCodes, OidcAuthorizationTransactions, OidcClients, Users,
        },
        sea_orm_active_enums::AuditType,
    },
};
use oceaniam_oidc::{
    AUTHORIZATION_FORM_MAX_BYTES, authorization_code_digest, verify_code_verifier_s256,
};
use sea_orm::{DatabaseTransaction, TransactionTrait};
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::ToSchema;
use utoipa_axum::{
    router::{UtoipaMethodRouter, UtoipaMethodRouterExt},
    routes,
};
use uuid::Uuid;

use crate::state::AppState;

use super::{
    authorize::{PreviewEnabled, apply_common_headers, local_response},
    login::is_unavailable,
};

/// Both issued tokens share a one-hour lifetime; neither is persisted.
const TOKEN_TTL_SECONDS: i64 = 3600;

const GRANT_TYPE_AUTHORIZATION_CODE: &str = "authorization_code";
const INVALID_REQUEST: &str = "invalid_request";
const INVALID_GRANT: &str = "invalid_grant";
const UNSUPPORTED_GRANT_TYPE: &str = "unsupported_grant_type";

/// Token endpoint parameters as deserialized from the form transport.
///
/// Fields are optional so missing parameters report the RFC 6749 §5.2 `invalid_request`
/// error; empty values are treated as omission. Deserialization is lossy (WHATWG form URL
/// decoding), with downstream exact-match validation as the security backstop. This type
/// deliberately omits `Debug`: it carries the plaintext code and verifier.
#[derive(Deserialize)]
struct RawTokenRequest {
    grant_type: Option<String>,
    code: Option<String>,
    redirect_uri: Option<String>,
    client_id: Option<String>,
    code_verifier: Option<String>,
}

/// RFC 6749 §5.1 successful token response. No refresh token is issued this slice.
#[derive(Serialize, ToSchema)]
struct OidcTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i64,
    id_token: String,
}

/// RFC 6749 §5.2 error response.
#[derive(Serialize, ToSchema)]
struct OidcTokenError {
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_description: Option<String>,
}

struct IssuedTokens {
    access_token: String,
    id_token: String,
}

enum TokenFailure {
    /// Request binding validation failed after the code was consumed; the caller commits the
    /// burn and answers with this RFC 6749 §5.2 error code.
    Protocol(&'static str),
    /// Signing or storage failed; the caller rolls back so the code survives for a retry.
    Internal,
}

/// Default-disabled OIDC Token Endpoint: exchanges an authorization code for tokens.
#[utoipa::path(
    post,
    path = "/oidc/{tenant_sqid}/token",
    tag = "Oidc",
    params(
        ("tenant_sqid" = String, Path, description = "Tenant Sqid"),
    ),
    request_body(
        content = String,
        content_type = "application/x-www-form-urlencoded",
        description = "Token parameters: `grant_type` (only `authorization_code`), `code`, `redirect_uri`, `client_id`, and the PKCE `code_verifier`; body-only, maximum 8 KiB"
    ),
    responses(
        (status = 200, description = "RFC 6749 5.1 token response: JWT access token and OIDC ID token, each with a one-hour lifetime; no refresh token is issued", body = OidcTokenResponse),
        (status = 400, description = "RFC 6749 5.2 error: `unsupported_grant_type`, `invalid_request`, or `invalid_grant`", body = OidcTokenError),
        (status = 404, description = "Preview disabled", body = String),
        (status = 413, description = "Form body exceeds 8 KiB", body = String),
        (status = 415, description = "Content type is not application/x-www-form-urlencoded", body = String),
        (status = 500, description = "Internal server error", body = String),
    ),
)]
#[tracing::instrument(level = "info", name = "oidc.token.post", skip_all)]
async fn post_oidc_token(
    _enabled: PreviewEnabled,
    Path(tenant_sqid): Path<String>,
    State(state): State<AppState>,
    form: Result<Form<RawTokenRequest>, FormRejection>,
) -> Response {
    let form = match form {
        Ok(Form(form)) => form,
        Err(rejection) => return token_form_rejection(rejection),
    };

    // Request-shape validation happens before any redemption work: an unsupported grant type
    // or a malformed request never touches the code.
    let Some(grant_type) = required(form.grant_type) else {
        return token_error(INVALID_REQUEST);
    };
    if grant_type != GRANT_TYPE_AUTHORIZATION_CODE {
        return token_error(UNSUPPORTED_GRANT_TYPE);
    }
    let (Some(code), Some(client_id), Some(redirect_uri), Some(code_verifier)) = (
        required(form.code),
        required(form.client_id),
        required(form.redirect_uri),
        required(form.code_verifier),
    ) else {
        return token_error(INVALID_REQUEST);
    };
    // The path tenant scopes the URL namespace only; the consumed code's transaction row
    // carries the authoritative tenant (issuer snapshot and signing keybox). A malformed
    // identifier is a malformed request.
    if tenant_sqid
        .parse::<Sqid>()
        .ok()
        .and_then(|sqid| Uuid::try_from(sqid).ok())
        .is_none()
    {
        return token_error(INVALID_REQUEST);
    }

    let code_digest = authorization_code_digest(&code);

    let transaction = match state.database.begin().await {
        Ok(transaction) => transaction,
        Err(error) => {
            error!(?error, "failed to begin OIDC token transaction");
            return local_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error");
        }
    };

    // Consume first, validate bindings second. An unknown, redeemed, or expired code burns
    // nothing, so the exchange fails without touching the database transaction further.
    let code_row =
        match OidcAuthorizationCodes::consume_authorization_code(code_digest, &transaction).await {
            Ok(code_row) => code_row,
            Err(error) => {
                if transaction.rollback().await.is_err() {
                    error!("failed to roll back OIDC token transaction");
                }
                if is_unavailable(&error) {
                    return token_error(INVALID_GRANT);
                }
                error!(?error, "failed to consume OIDC authorization code");
                return local_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error");
            }
        };

    let outcome = exchange_consumed_code(
        &state,
        &transaction,
        code_row,
        &client_id,
        &redirect_uri,
        &code_verifier,
    )
    .await;

    match outcome {
        Ok(issued) => {
            if transaction.commit().await.is_err() {
                // Commit outcome can be ambiguous. Never retry and never report success.
                error!("failed to commit OIDC token transaction");
                return local_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error");
            }
            token_response(issued)
        }
        Err(TokenFailure::Protocol(error_code)) => {
            // Fail closed: validation failures commit the burn (redeemed_at is persisted) so
            // client, redirect, and PKCE mismatches cannot be probed online.
            if transaction.commit().await.is_err() {
                error!("failed to commit the OIDC token code burn");
                return local_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error");
            }
            token_error(error_code)
        }
        Err(TokenFailure::Internal) => {
            if transaction.rollback().await.is_err() {
                error!("failed to roll back OIDC token transaction");
            }
            local_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
        }
    }
}

/// Binding validation, claims assembly, signing, and the issuance audit for a consumed code.
///
/// Every protocol failure leaves the burn for the caller to commit; every internal failure
/// leaves nothing behind.
async fn exchange_consumed_code(
    state: &AppState,
    transaction: &DatabaseTransaction,
    code_row: oidc_authorization_codes::Model,
    client_id: &str,
    redirect_uri: &str,
    code_verifier: &str,
) -> Result<IssuedTokens, TokenFailure> {
    let snapshot = OidcAuthorizationTransactions::get_token_authorization_transaction(
        code_row.transaction_id,
        transaction,
    )
    .await
    .map_err(|error| {
        // The consumed code's foreign key guarantees the transaction row.
        error!(
            ?error,
            "failed to load OIDC authorization transaction for token exchange"
        );
        TokenFailure::Internal
    })?;

    // A code is issued only after authentication terminates the transaction, so both fields
    // are always present; their absence is an internal inconsistency, not a client error.
    let (Some(subject_id), Some(authenticated_at)) =
        (snapshot.subject_id, snapshot.authenticated_at)
    else {
        error!("consumed OIDC authorization code references an unauthenticated transaction");
        return Err(TokenFailure::Internal);
    };

    let client = OidcClients::get_client_by_id(snapshot.oidc_client_id, transaction)
        .await
        .map_err(|error| {
            // The owner cascade removes the transaction (and its codes) with the client, so a
            // consumed code always resolves its client row.
            error!(?error, "failed to load OIDC client for token exchange");
            TokenFailure::Internal
        })?;

    // RFC 6749 §4.1.3: the token request's client_id and redirect_uri must match the values
    // bound at authorization time; comparison is byte-exact (§4.1.3 MUST).
    if client_id != client.client_id {
        return Err(TokenFailure::Protocol(INVALID_GRANT));
    }
    if redirect_uri != snapshot.redirect_uri {
        return Err(TokenFailure::Protocol(INVALID_GRANT));
    }
    if verify_code_verifier_s256(code_verifier, &snapshot.code_challenge).is_err() {
        return Err(TokenFailure::Protocol(INVALID_GRANT));
    }

    // The transaction records the internal `users.id`; the JWT subject is the stable external
    // `users.oidc_sub` (see `Claim`), so the user row resolves it.
    let user = Users::get_user_of_application(snapshot.application_id, subject_id, transaction)
        .await
        .map_err(|error| {
            if is_unavailable(&error) {
                // The subject's user row vanished after the code was issued; the grant can no
                // longer be honored, so it fails closed like any other invalid grant.
                TokenFailure::Protocol(INVALID_GRANT)
            } else {
                error!(?error, "failed to load OIDC token subject");
                TokenFailure::Internal
            }
        })?;

    let now = chrono::Utc::now().timestamp();
    let access_claim = Claim {
        sub: user.oidc_sub,
        exp: now + TOKEN_TTL_SECONDS,
        iat: now,
        iss: Some(snapshot.issuer.clone()),
        aud: Some(vec![client.client_id.clone()]),
        jti: Uuid::now_v7(),
    };
    let id_claim = IdTokenClaim {
        iss: snapshot.issuer.clone(),
        sub: user.oidc_sub,
        aud: vec![client.client_id.clone()],
        exp: now + TOKEN_TTL_SECONDS,
        iat: now,
        nonce: snapshot.nonce.clone(),
        auth_time: authenticated_at.timestamp(),
    };

    let access_token = state
        .keyboxes
        .clone()
        .sign_assembled_jwt(snapshot.tenant_id, access_claim)
        .await
        .map_err(|error| {
            error!(?error, "failed to sign OIDC access token");
            TokenFailure::Internal
        })?
        .jwt;
    let id_token = state
        .keyboxes
        .clone()
        .sign_assembled_jwt(snapshot.tenant_id, id_claim)
        .await
        .map_err(|error| {
            error!(?error, "failed to sign OIDC ID token");
            TokenFailure::Internal
        })?
        .jwt;

    let payload = AuditPayload::from(IssueTokenPayload {
        application_id: snapshot.application_id,
        transaction_id: snapshot.id,
        subject_id,
    })
    .into_json()
    .map_err(|error| {
        error!(
            ?error,
            "failed to serialize OIDC token issuance audit payload"
        );
        TokenFailure::Internal
    })?;
    Audits::insert_audit_event(
        Uuid::now_v7(),
        AuditType::OidcIssueToken,
        payload,
        transaction,
    )
    .await
    .map_err(|error| {
        error!(?error, "failed to insert OIDC token issuance audit");
        TokenFailure::Internal
    })?;

    Ok(IssuedTokens {
        access_token,
        id_token,
    })
}

/// Empty values are treated as omission, so a required parameter must be present and non-empty.
fn required(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.is_empty())
}

fn token_response(issued: IssuedTokens) -> Response {
    let mut response = Json(OidcTokenResponse {
        access_token: issued.access_token,
        token_type: "Bearer".to_owned(),
        expires_in: TOKEN_TTL_SECONDS,
        id_token: issued.id_token,
    })
    .into_response();
    apply_common_headers(&mut response);
    response
}

fn token_error(error: &'static str) -> Response {
    let mut response = (
        StatusCode::BAD_REQUEST,
        Json(OidcTokenError {
            error: error.to_owned(),
            error_description: None,
        }),
    )
        .into_response();
    apply_common_headers(&mut response);
    response
}

/// Transport rejections keep their status (the family's plain-text shape); only a well-formed
/// form transport with unusable content degrades to the RFC 6749 §5.2 `invalid_request`.
fn token_form_rejection(rejection: FormRejection) -> Response {
    match rejection.status() {
        StatusCode::PAYLOAD_TOO_LARGE => local_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            "token request body is too large",
        ),
        StatusCode::UNSUPPORTED_MEDIA_TYPE => local_response(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "token request must be form-encoded",
        ),
        _ => token_error(INVALID_REQUEST),
    }
}

pub(super) fn routes() -> UtoipaMethodRouter<AppState, Infallible> {
    routes!(post_oidc_token)
        .map(|methods| methods.route_layer(DefaultBodyLimit::max(AUTHORIZATION_FORM_MAX_BYTES)))
}
