//! Transaction-bound password login for the OIDC authorization preview.
//!
//! This endpoint advances an authorization transaction `pending -> authenticated`. It reuses
//! the entry endpoint's transport boundary (preview gate, bounded lossy form parsing, no query
//! parameters on POST), the browser-binding cookie and CSRF digests, the ordered shared-lock
//! registration revalidation, and the uniform anti-enumeration failure message. It never logs
//! the identifier, password, or per-branch failure causes, and it never echoes state, nonce,
//! or redirect values.

use std::convert::Infallible;

use axum::{
    extract::{DefaultBodyLimit, FromRequestParts, Path, State},
    http::{
        HeaderValue, StatusCode,
        header::{CONTENT_SECURITY_POLICY, CONTENT_TYPE},
        request::Parts,
    },
    response::{Html, IntoResponse, Response},
};
use axum_extra::extract::{Form, FormRejection, cookie::CookieJar};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use oceaniam_audit::types::{AuditPayload, OidcAuthenticatePayload};
use oceaniam_common::sqid::Sqid;
use oceaniam_database::{
    config::application::ApplicationConfiguration,
    error::Error as DatabaseError,
    helper::{
        audits::AuditsHelper,
        oidc_authorization_entry::{AuthorizationEntryLockResult, OidcAuthorizationEntryHelper},
        oidc_authorization_transactions::{
            AuthenticateAuthorizationTransactionInput, OidcAuthorizationTransactionsHelper,
        },
        subjects::SubjectsHelper,
    },
    model::{
        oidc_authorization_transactions,
        prelude::{Audits, OidcAuthorizationTransactions, OidcClients, Subjects},
        sea_orm_active_enums::AuditType,
    },
};
use oceaniam_oidc::{
    AUTHORIZATION_FORM_MAX_BYTES, ParsedLoginForm, RedirectUriPolicy,
    authorization_browser_binding_digest, authorization_csrf_digest, validate_redirect_uri,
};
use sea_orm::{DatabaseTransaction, TransactionTrait};
use tracing::error;
use utoipa_axum::{
    router::{UtoipaMethodRouter, UtoipaMethodRouterExt},
    routes,
};
use uuid::Uuid;

use crate::state::{AppState, applications::UserIdentifier};

use super::authorize::{
    AUTHORIZATION_FORM_CONTENT_SECURITY_POLICY_VALUE, AuthorizationTarget, COOKIE_PREFIX,
    PreviewEnabled, apply_common_headers, form_rejection, local_response,
};

const TRANSACTION_UNAVAILABLE_HTML: &str = "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Sign-in unavailable</title></head><body><main><h1>Sign-in unavailable</h1><p>The sign-in session is no longer available. Restart the authorization request.</p></main></body></html>";
const SUCCESS_HTML: &str = "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Sign-in recorded</title></head><body><main><h1>Sign-in recorded</h1><p>authentication recorded; authorization continuation is not yet available</p></main></body></html>";

struct AuthorizationLoginPath {
    tenant_id: Uuid,
    transaction_id: Uuid,
}

impl FromRequestParts<AppState> for AuthorizationLoginPath {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Path((tenant_sqid, transaction_sqid)) =
            Path::<(String, String)>::from_request_parts(parts, state)
                .await
                .map_err(|_| local_response(StatusCode::BAD_REQUEST, "invalid login path"))?;
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

/// Default-disabled OIDC authorization login: password-authenticates a pending transaction.
#[utoipa::path(
    post,
    path = "/oidc/{tenant_sqid}/authorize/{transaction_sqid}/login",
    tag = "Oidc",
    params(
        ("tenant_sqid" = String, Path, description = "Tenant Sqid"),
        ("transaction_sqid" = String, Path, description = "Authorization transaction Sqid"),
    ),
    request_body(
        content = String,
        content_type = "application/x-www-form-urlencoded",
        description = "Login parameters: `identifier` (email or phone), `password`, and the transaction `csrf` secret; body-only, maximum 8 KiB"
    ),
    responses(
        (status = 200, description = "Authentication recorded; authorization continuation is not yet available", body = String, content_type = "text/html"),
        (status = 400, description = "Malformed login request or invalidated registration", body = String),
        (status = 401, description = "Indistinguishable credential failure page", body = String, content_type = "text/html"),
        (status = 404, description = "Preview disabled, or the transaction binding is unavailable", body = String),
        (status = 405, description = "Only POST is supported", body = String),
        (status = 413, description = "Form body exceeds 8 KiB", body = String),
        (status = 414, description = "Request target exceeds 8 KiB", body = String),
        (status = 415, description = "Content type is not application/x-www-form-urlencoded", body = String),
        (status = 500, description = "Internal server error", body = String),
    ),
)]
#[tracing::instrument(level = "info", name = "oidc.authorization_login.post", skip_all)]
async fn post_authorization_login(
    _enabled: PreviewEnabled,
    _target: AuthorizationTarget,
    AuthorizationLoginPath {
        tenant_id,
        transaction_id,
    }: AuthorizationLoginPath,
    State(state): State<AppState>,
    cookies: CookieJar,
    form: Result<Form<ParsedLoginForm>, FormRejection>,
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

    let snapshot = match OidcAuthorizationTransactions::get_login_authorization_transaction(
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
                "failed to read OIDC authorization transaction for login"
            );
            return local_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error");
        }
    };

    let transaction = match state.database.begin().await {
        Ok(transaction) => transaction,
        Err(error) => {
            error!(
                ?error,
                "failed to begin OIDC authorization-login transaction"
            );
            return local_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error");
        }
    };

    let outcome = authenticate_in_transaction(
        &state,
        &transaction,
        tenant_id,
        &snapshot,
        browser_binding_digest,
        csrf_digest,
        &form,
    )
    .await;

    if let Err(failure) = outcome {
        if transaction.rollback().await.is_err() {
            error!("failed to roll back OIDC authorization-login transaction");
        }
        return failure.into_response();
    }

    if transaction.commit().await.is_err() {
        // Commit outcome can be ambiguous. Never retry and never report success.
        error!("failed to commit OIDC authorization-login transaction");
        return local_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error");
    }

    html_response(StatusCode::OK, SUCCESS_HTML)
}

/// Live revalidation, credential verification, state transition, and the success-only audit in
/// one short transaction. Any failure rolls back without a transition or partial writes.
async fn authenticate_in_transaction(
    state: &AppState,
    transaction: &DatabaseTransaction,
    tenant_id: Uuid,
    snapshot: &oidc_authorization_transactions::Model,
    browser_binding_digest: [u8; 32],
    csrf_digest: [u8; 32],
    form: &ParsedLoginForm,
) -> Result<(), LoginFailure> {
    let locked = match OidcClients::lock_authorization_login_registration(
        tenant_id,
        snapshot.application_id,
        snapshot.oidc_client_id,
        &snapshot.redirect_uri,
        transaction,
    )
    .await
    .map_err(|_| LoginFailure::Internal)?
    {
        AuthorizationEntryLockResult::TenantUnavailable => {
            return Err(LoginFailure::RegistrationUnavailable(
                StatusCode::NOT_FOUND,
                "tenant not found",
            ));
        }
        AuthorizationEntryLockResult::RelationshipUnavailable => {
            return Err(LoginFailure::RegistrationUnavailable(
                StatusCode::BAD_REQUEST,
                "invalid authorization request",
            ));
        }
        AuthorizationEntryLockResult::Locked(locked) => locked,
    };

    let application_configuration: ApplicationConfiguration =
        serde_json::from_value(locked.application_configuration)
            .map_err(|_| LoginFailure::Internal)?;
    let redirect_policy = if application_configuration
        .oidc
        .allow_insecure_loopback_redirect_uris
    {
        RedirectUriPolicy::web_with_insecure_loopback()
    } else {
        RedirectUriPolicy::web()
    };
    if validate_redirect_uri(&snapshot.redirect_uri, redirect_policy).is_err() {
        return Err(LoginFailure::RegistrationUnavailable(
            StatusCode::BAD_REQUEST,
            "invalid authorization request",
        ));
    }

    let subject_id =
        verify_login_credentials(state, transaction, snapshot.application_id, form).await?;

    OidcAuthorizationTransactions::authenticate_authorization_transaction(
        AuthenticateAuthorizationTransactionInput {
            id: snapshot.id,
            browser_binding_digest,
            csrf_digest,
            expected_revision: snapshot.revision,
            subject_id,
        },
        transaction,
    )
    .await
    .map_err(|error| {
        if is_unavailable(&error) {
            LoginFailure::TransactionUnavailable
        } else {
            LoginFailure::Internal
        }
    })?;

    let payload = AuditPayload::from(OidcAuthenticatePayload {
        application_id: snapshot.application_id,
        transaction_id: snapshot.id,
        subject_id,
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

    Ok(())
}

/// Verifies the submitted credentials strictly within the transaction snapshot's Application.
///
/// Unknown user, wrong password, inactive subject, and MFA-registered subjects all return the
/// same indistinguishable failure; the cause is deliberately never logged.
async fn verify_login_credentials(
    state: &AppState,
    transaction: &DatabaseTransaction,
    application_id: Uuid,
    form: &ParsedLoginForm,
) -> Result<Uuid, LoginFailure> {
    let (Some(identifier), Some(password)) = (form.identifier(), form.password()) else {
        return Err(LoginFailure::LoginFailed);
    };
    let user_identifier = if identifier.contains('@') {
        UserIdentifier::Email(identifier.to_owned())
    } else {
        UserIdentifier::Phone(identifier.to_owned())
    };

    let user = state
        .applications
        .find_user_by(application_id, user_identifier)
        .await
        .map_err(|_| LoginFailure::LoginFailed)?;
    let password_matches = state
        .credentials
        .verify_password(user.id, password)
        .await
        .map_err(|_| LoginFailure::LoginFailed)?;
    if !password_matches {
        return Err(LoginFailure::LoginFailed);
    }
    // Lazy expiration rejection, checked only after the password matches (see the committed
    // application signin flow), so the generic failure cannot probe account expiration.
    Subjects::ensure_subject_active(user.id, transaction)
        .await
        .map_err(|_| LoginFailure::LoginFailed)?;
    // MFA-registered subjects cannot complete this slice; the rejection is indistinguishable
    // from a wrong password.
    let mfa_registered = state
        .credentials
        .has_totp(user.id)
        .await
        .map_err(|_| LoginFailure::LoginFailed)?;
    if mfa_registered {
        return Err(LoginFailure::LoginFailed);
    }

    Ok(user.id)
}

enum LoginFailure {
    TransactionUnavailable,
    RegistrationUnavailable(StatusCode, &'static str),
    LoginFailed,
    Internal,
}

impl LoginFailure {
    fn into_response(self) -> Response {
        match self {
            Self::TransactionUnavailable => transaction_unavailable_response(),
            Self::RegistrationUnavailable(status, message) => local_response(status, message),
            Self::LoginFailed => login_failed_response(),
            Self::Internal => {
                local_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
            }
        }
    }
}

fn is_unavailable(error: &DatabaseError) -> bool {
    matches!(
        error,
        DatabaseError::CustomMessage { code, .. } if *code == StatusCode::NOT_FOUND.as_u16()
    )
}

fn transaction_unavailable_response() -> Response {
    html_response(StatusCode::NOT_FOUND, TRANSACTION_UNAVAILABLE_HTML)
}

fn login_failed_response() -> Response {
    let html = format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Sign-in failed</title></head><body><main><h1>Sign-in failed</h1><p>{}</p></main></body></html>",
        oceaniam_common::consts::USER_LOGIN_FAILED_MSG,
    );
    html_response(StatusCode::UNAUTHORIZED, &html)
}

fn html_response(status: StatusCode, html: &str) -> Response {
    let mut response = (status, Html(html.to_owned())).into_response();
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    response.headers_mut().insert(
        CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(AUTHORIZATION_FORM_CONTENT_SECURITY_POLICY_VALUE),
    );
    apply_common_headers(&mut response);
    response
}

pub(super) fn routes() -> UtoipaMethodRouter<AppState, Infallible> {
    routes!(post_authorization_login)
        .map(|methods| methods.route_layer(DefaultBodyLimit::max(AUTHORIZATION_FORM_MAX_BYTES)))
}
