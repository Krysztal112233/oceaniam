use std::convert::Infallible;

use axum::{
    extract::{DefaultBodyLimit, FromRequestParts, OriginalUri, Path, State},
    http::{
        HeaderValue, Method, StatusCode,
        header::{
            self, ALLOW, CACHE_CONTROL, CONTENT_SECURITY_POLICY, CONTENT_TYPE, InvalidHeaderValue,
            LOCATION, PRAGMA, REFERRER_POLICY, SET_COOKIE,
        },
        request::Parts,
    },
    response::{Html, IntoResponse, Response},
};
use axum_extra::extract::{
    Form, FormRejection, Query, QueryRejection,
    cookie::{Cookie, SameSite},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use oceaniam_common::{config::PublicBaseUrl, sqid::Sqid};
use oceaniam_database::{
    config::application::ApplicationConfiguration,
    helper::{
        oidc_authorization_entry::{
            AuthorizationEntryLockResult, AuthorizationEntryResolution,
            OidcAuthorizationEntryHelper,
        },
        oidc_authorization_transactions::{
            CreateAuthorizationTransactionInput, OidcAuthorizationTransactionsHelper,
        },
    },
    model::{
        oidc_authorization_transactions,
        prelude::{OidcAuthorizationTransactions, OidcClients},
    },
};
use oceaniam_oidc::{
    AUTHORIZATION_FORM_MAX_BYTES, AuthorizationProtocolError, ParsedAuthorizationRequest,
    RawAuthorizationRequest, RedirectUriPolicy, authorization_browser_binding_digest,
    authorization_csrf_digest, validate_redirect_uri,
};
use rand::{RngCore, rngs::OsRng};
use sea_orm::{DatabaseTransaction, TransactionTrait};
use time::Duration;
use tracing::error;
use url::form_urlencoded;
use utoipa_axum::{
    router::{UtoipaMethodRouter, UtoipaMethodRouterExt},
    routes,
};
use uuid::Uuid;

use crate::state::AppState;

pub(super) const REQUEST_TARGET_MAX_BYTES: usize = 8 * 1024;
const CACHE_CONTROL_VALUE: &str = "no-store";
const PRAGMA_VALUE: &str = "no-cache";
const REFERRER_POLICY_VALUE: &str = "no-referrer";
/// HTML pages that carry the transaction-bound login form (and its outcome pages) may post
/// back to their own origin only; scripts, frames, and external resources remain forbidden.
pub(super) const AUTHORIZATION_FORM_CONTENT_SECURITY_POLICY_VALUE: &str = "default-src 'none'; base-uri 'none'; connect-src 'none'; font-src 'none'; form-action 'self'; frame-ancestors 'none'; img-src 'none'; media-src 'none'; object-src 'none'; script-src 'none'; style-src 'none'";
pub(super) const COOKIE_PREFIX: &str = "oceaniam_oidc_binding_";

pub(super) struct PreviewEnabled;

impl FromRequestParts<AppState> for PreviewEnabled {
    type Rejection = Response;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if state.oidc_authorization_entry_preview_enabled {
            Ok(Self)
        } else {
            Err(local_response(StatusCode::NOT_FOUND, "not found"))
        }
    }
}

pub(super) struct AuthorizationTarget;

impl FromRequestParts<AppState> for AuthorizationTarget {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let uri = match OriginalUri::from_request_parts(parts, state).await {
            Ok(OriginalUri(uri)) => uri,
            Err(infallible) => match infallible {},
        };
        if uri.to_string().len() > REQUEST_TARGET_MAX_BYTES {
            return Err(local_response(
                StatusCode::URI_TOO_LONG,
                "authorization request target is too long",
            ));
        }

        match parts.method {
            Method::GET | Method::POST | Method::HEAD => {}
            _ => {
                return Err(local_response(
                    StatusCode::METHOD_NOT_ALLOWED,
                    "method not allowed",
                ));
            }
        }
        if parts.method == Method::POST && uri.query().is_some() {
            return Err(local_response(
                StatusCode::BAD_REQUEST,
                "POST authorization requests must not include query parameters",
            ));
        }

        Ok(Self)
    }
}

struct AuthorizationTenantPath(String);

impl FromRequestParts<AppState> for AuthorizationTenantPath {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Path::<String>::from_request_parts(parts, state)
            .await
            .map(|Path(value)| Self(value))
            .map_err(|_| local_response(StatusCode::BAD_REQUEST, "invalid tenant identifier"))
    }
}

pub(super) fn form_rejection(rejection: FormRejection) -> Response {
    let status = match rejection.status() {
        StatusCode::PAYLOAD_TOO_LARGE => StatusCode::PAYLOAD_TOO_LARGE,
        StatusCode::UNSUPPORTED_MEDIA_TYPE => StatusCode::UNSUPPORTED_MEDIA_TYPE,
        _ => StatusCode::BAD_REQUEST,
    };
    local_response(status, "invalid authorization request body")
}

/// Default-disabled OIDC Authorization Endpoint preview using query parameters.
#[utoipa::path(
    get,
    path = "/oidc/{tenant_sqid}/authorize",
    tag = "Oidc",
    params(
        ("tenant_sqid" = String, Path, description = "Tenant Sqid"),
        ("response_type" = Option<String>, Query, description = "Only `code` is supported"),
        ("client_id" = Option<String>, Query, description = "Opaque registered client identifier"),
        ("redirect_uri" = Option<String>, Query, description = "Exact registered redirect URI"),
        ("scope" = Option<String>, Query, description = "Only `openid` is supported"),
        ("state" = Option<String>, Query, description = "Required opaque client state"),
        ("nonce" = Option<String>, Query, description = "Optional OIDC nonce"),
        ("code_challenge" = Option<String>, Query, description = "Required canonical PKCE S256 challenge"),
        ("code_challenge_method" = Option<String>, Query, description = "Must be `S256`"),
        ("response_mode" = Option<String>, Query, description = "Only omitted or `query` is supported"),
    ),
    responses(
        (status = 200, description = "Static sign-in-unavailable preview; creates a ten-minute request snapshot", body = String, content_type = "text/html"),
        (status = 303, description = "Protocol error redirected only to a live trusted callback"),
        (status = 400, description = "Malformed or untrusted authorization request", body = String),
        (status = 404, description = "Preview disabled or tenant unavailable", body = String),
        (status = 414, description = "Request target exceeds 8 KiB", body = String),
        (status = 500, description = "Internal server error", body = String),
    ),
)]
#[tracing::instrument(level = "info", name = "oidc.authorization_entry.get", skip_all)]
async fn get_authorization_entry(
    _enabled: PreviewEnabled,
    _target: AuthorizationTarget,
    AuthorizationTenantPath(tenant_sqid): AuthorizationTenantPath,
    State(state): State<AppState>,
    query: Result<Query<RawAuthorizationRequest>, QueryRejection>,
) -> Response {
    let raw = match query {
        Ok(Query(raw)) => raw,
        Err(_) => {
            return local_response(StatusCode::BAD_REQUEST, "malformed authorization request");
        }
    };
    handle_authorization_entry(state, tenant_sqid, ParsedAuthorizationRequest::from(raw)).await
}

/// Default-disabled OIDC Authorization Endpoint preview using a form body.
#[utoipa::path(
    post,
    path = "/oidc/{tenant_sqid}/authorize",
    tag = "Oidc",
    params(
        ("tenant_sqid" = String, Path, description = "Tenant Sqid"),
    ),
    request_body(
        content = String,
        content_type = "application/x-www-form-urlencoded",
        description = "Authorization parameters; same profile as GET, body-only, maximum 8 KiB"
    ),
    responses(
        (status = 200, description = "Static sign-in-unavailable preview; creates a ten-minute request snapshot", body = String, content_type = "text/html"),
        (status = 303, description = "Protocol error redirected only to a live trusted callback"),
        (status = 400, description = "Malformed, mixed-query, or untrusted authorization request", body = String),
        (status = 404, description = "Preview disabled or tenant unavailable", body = String),
        (status = 413, description = "Form body exceeds 8 KiB", body = String),
        (status = 414, description = "Request target exceeds 8 KiB", body = String),
        (status = 415, description = "Content type is not application/x-www-form-urlencoded", body = String),
        (status = 500, description = "Internal server error", body = String),
    ),
)]
#[tracing::instrument(level = "info", name = "oidc.authorization_entry.post", skip_all)]
async fn post_authorization_entry(
    _enabled: PreviewEnabled,
    _target: AuthorizationTarget,
    AuthorizationTenantPath(tenant_sqid): AuthorizationTenantPath,
    State(state): State<AppState>,
    form: Result<Form<RawAuthorizationRequest>, FormRejection>,
) -> Response {
    let raw = match form {
        Ok(Form(raw)) => raw,
        Err(rejection) => return form_rejection(rejection),
    };
    handle_authorization_entry(state, tenant_sqid, ParsedAuthorizationRequest::from(raw)).await
}

#[tracing::instrument(level = "info", name = "oidc.authorization_entry.head", skip_all)]
async fn head_authorization_entry(
    _enabled: PreviewEnabled,
    _target: AuthorizationTarget,
) -> Response {
    let mut response = local_response(StatusCode::METHOD_NOT_ALLOWED, "method not allowed");
    response
        .headers_mut()
        .insert(ALLOW, HeaderValue::from_static("GET, POST"));
    response
}

pub(super) fn routes() -> UtoipaMethodRouter<AppState, Infallible> {
    routes!(get_authorization_entry, post_authorization_entry).map(|methods| {
        methods
            .head(head_authorization_entry)
            .route_layer(DefaultBodyLimit::max(AUTHORIZATION_FORM_MAX_BYTES))
    })
}

async fn handle_authorization_entry(
    state: AppState,
    tenant_sqid: String,
    parsed: ParsedAuthorizationRequest,
) -> Response {
    // OIDC Core 1.0 §5: an unsupported Response Mode gets a bare HTTP 400 without Error
    // Response parameters (the mode would encode them), so it is rejected before any
    // callback-trust work and never becomes a redirect.
    if parsed.has_unsupported_response_mode() {
        return local_response(StatusCode::BAD_REQUEST, "unsupported response mode");
    }

    let supplied_sqid = match tenant_sqid.parse::<Sqid>() {
        Ok(sqid) => sqid,
        Err(_) => {
            return local_response(StatusCode::BAD_REQUEST, "invalid tenant identifier");
        }
    };
    let tenant_id = match Uuid::try_from(supplied_sqid) {
        Ok(tenant_id) => tenant_id,
        Err(_) => {
            return local_response(StatusCode::BAD_REQUEST, "invalid tenant identifier");
        }
    };
    let canonical_tenant_sqid = Sqid::from(tenant_id);

    // Empty values were normalized to omission during deserialization. Retain empty lookup
    // sentinels so the transaction can resolve tenant availability before rejecting
    // client/callback fields.
    let client_id = parsed.client_id().unwrap_or_default().to_owned();
    let redirect_uri = parsed.redirect_uri().unwrap_or_default().to_owned();
    let state_parameter = parsed.state().map(ToOwned::to_owned);
    let issuer = tenant_issuer(&state.public_base_url, &canonical_tenant_sqid);

    let transaction = match state.database.begin().await {
        Ok(transaction) => transaction,
        Err(_) => {
            error!("failed to begin OIDC authorization-entry transaction");
            return local_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error");
        }
    };

    let outcome = create_snapshot_in_transaction(
        &transaction,
        tenant_id,
        issuer,
        client_id,
        redirect_uri,
        state_parameter,
        parsed,
    )
    .await;

    let prepared = match outcome {
        Ok(prepared) => prepared,
        Err(failure) => {
            if transaction.rollback().await.is_err() {
                error!("failed to roll back OIDC authorization-entry transaction");
            }
            return failure.into_response();
        }
    };

    let response = match success_response(&prepared) {
        Ok(response) => response,
        Err(error) => {
            error!(
                ?error,
                "failed to construct an OIDC authorization-entry response header"
            );
            let trusted_callback = prepared.trusted_callback.clone();
            if transaction.rollback().await.is_err() {
                error!("failed to roll back OIDC authorization-entry response preparation");
            }
            return internal_response(Some(trusted_callback));
        }
    };
    let trusted_callback = prepared.trusted_callback.clone();

    if transaction.commit().await.is_err() {
        // Commit outcome can be ambiguous. Never retry and never expose the prepared success
        // cookie; a trusted callback receives only a standard server_error.
        error!("failed to commit OIDC authorization-entry transaction");
        return internal_response(Some(trusted_callback));
    }

    response
}

async fn create_snapshot_in_transaction(
    transaction: &DatabaseTransaction,
    tenant_id: Uuid,
    issuer: String,
    client_id: String,
    redirect_uri: String,
    state_parameter: Option<String>,
    parsed: ParsedAuthorizationRequest,
) -> Result<PreparedSuccess, EntryFailure> {
    let candidate = match OidcClients::resolve_authorization_entry_candidate(
        tenant_id,
        &client_id,
        transaction,
    )
    .await
    .map_err(|_| EntryFailure::Internal(None))?
    {
        AuthorizationEntryResolution::TenantUnavailable => {
            return Err(EntryFailure::Local(
                StatusCode::NOT_FOUND,
                "tenant not found",
            ));
        }
        AuthorizationEntryResolution::ClientUnavailable => {
            return Err(EntryFailure::Local(
                StatusCode::BAD_REQUEST,
                "invalid authorization request",
            ));
        }
        AuthorizationEntryResolution::Candidate(candidate) => candidate,
    };

    let locked = match OidcClients::lock_authorization_entry_registration(
        tenant_id,
        candidate,
        &client_id,
        &redirect_uri,
        transaction,
    )
    .await
    .map_err(|_| EntryFailure::Internal(None))?
    {
        AuthorizationEntryLockResult::TenantUnavailable => {
            return Err(EntryFailure::Local(
                StatusCode::NOT_FOUND,
                "tenant not found",
            ));
        }
        AuthorizationEntryLockResult::RelationshipUnavailable => {
            return Err(EntryFailure::Local(
                StatusCode::BAD_REQUEST,
                "invalid authorization request",
            ));
        }
        AuthorizationEntryLockResult::Locked(locked) => locked,
    };

    let application_configuration: ApplicationConfiguration =
        serde_json::from_value(locked.application_configuration)
            .map_err(|_| EntryFailure::Internal(None))?;
    let redirect_policy = if application_configuration
        .oidc
        .allow_insecure_loopback_redirect_uris
    {
        RedirectUriPolicy::web_with_insecure_loopback()
    } else {
        RedirectUriPolicy::web()
    };
    if validate_redirect_uri(&redirect_uri, redirect_policy).is_err() {
        return Err(EntryFailure::Local(
            StatusCode::BAD_REQUEST,
            "invalid authorization request",
        ));
    }

    // Exact redirect registration and its governing Application policy are now protected by the
    // ordered shared locks, so protocol errors may use this callback.
    let trusted_callback = TrustedCallback {
        redirect_uri: redirect_uri.clone(),
        state: state_parameter,
    };
    let request = parsed
        .into_authorization_request()
        .map_err(|error| EntryFailure::Protocol {
            error,
            callback: trusted_callback.clone(),
        })?;

    let mut browser_binding = [0_u8; 32];
    let mut csrf = [0_u8; 32];
    let mut rng = OsRng;
    rng.fill_bytes(&mut browser_binding);
    rng.fill_bytes(&mut csrf);

    let snapshot = OidcAuthorizationTransactions::create_authorization_transaction(
        CreateAuthorizationTransactionInput {
            tenant_id,
            application_id: locked.application_id,
            oidc_client_id: locked.oidc_client_id,
            issuer,
            redirect_uri,
            state: request.state().to_owned(),
            nonce: request.nonce().map(ToOwned::to_owned),
            code_challenge: request.code_challenge().to_owned(),
            browser_binding_digest: authorization_browser_binding_digest(&browser_binding),
            csrf_digest: authorization_csrf_digest(&csrf),
        },
        transaction,
    )
    .await
    .map_err(|_| EntryFailure::Internal(Some(trusted_callback.clone())))?;

    Ok(PreparedSuccess {
        snapshot,
        canonical_tenant_sqid: Sqid::from(tenant_id).to_string(),
        browser_binding,
        csrf,
        trusted_callback,
    })
}

struct PreparedSuccess {
    snapshot: oidc_authorization_transactions::Model,
    canonical_tenant_sqid: String,
    browser_binding: [u8; 32],
    csrf: [u8; 32],
    trusted_callback: TrustedCallback,
}

#[derive(Clone)]
struct TrustedCallback {
    redirect_uri: String,
    state: Option<String>,
}

enum EntryFailure {
    Local(StatusCode, &'static str),
    Protocol {
        error: AuthorizationProtocolError,
        callback: TrustedCallback,
    },
    Internal(Option<TrustedCallback>),
}

impl EntryFailure {
    fn into_response(self) -> Response {
        match self {
            Self::Local(status, message) => local_response(status, message),
            Self::Protocol { error, callback } => trusted_error_response(&callback, error)
                .unwrap_or_else(invalid_trusted_callback_response),
            Self::Internal(callback) => internal_response(callback),
        }
    }
}

fn internal_response(callback: Option<TrustedCallback>) -> Response {
    callback
        .as_ref()
        .map(|callback| {
            match trusted_error_response(callback, AuthorizationProtocolError::ServerError) {
                Ok(response) => response,
                Err(error) => invalid_trusted_callback_response(error),
            }
        })
        .unwrap_or_else(|| {
            local_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
        })
}

fn tenant_issuer(public_base_url: &PublicBaseUrl, tenant_sqid: &Sqid) -> String {
    let mut issuer = public_base_url.as_url().clone();
    issuer.set_path(&format!("/oidc/{tenant_sqid}"));
    issuer.to_string()
}

fn trusted_error_response(
    callback: &TrustedCallback,
    error: AuthorizationProtocolError,
) -> Result<Response, InvalidHeaderValue> {
    let mut query = form_urlencoded::Serializer::new(String::new());
    query.append_pair("error", error.as_str());
    if let Some(state) = callback.state.as_deref() {
        query.append_pair("state", state);
    }

    let mut location = callback.redirect_uri.clone();
    location.push(if location.contains('?') { '&' } else { '?' });
    location.push_str(&query.finish());
    let location = HeaderValue::from_str(&location)?;

    let mut response = StatusCode::SEE_OTHER.into_response();
    response.headers_mut().insert(LOCATION, location);
    apply_common_headers(&mut response);
    Ok(response)
}

fn invalid_trusted_callback_response(error: InvalidHeaderValue) -> Response {
    error!(
        ?error,
        "trusted OIDC callback produced an invalid Location header"
    );
    local_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
}

fn success_response(prepared: &PreparedSuccess) -> Result<Response, InvalidHeaderValue> {
    let transaction_sqid = Sqid::from(prepared.snapshot.id).to_string();
    let browser_binding = URL_SAFE_NO_PAD.encode(prepared.browser_binding);
    let csrf = URL_SAFE_NO_PAD.encode(prepared.csrf);
    let cookie_name = format!("{COOKIE_PREFIX}{transaction_sqid}");
    let cookie_path = format!("/oidc/{}", prepared.canonical_tenant_sqid);
    let cookie = Cookie::build((cookie_name, browser_binding))
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(prepared.snapshot.issuer.starts_with("https://"))
        .path(cookie_path)
        .max_age(Duration::seconds(600))
        .build();
    let cookie = HeaderValue::from_str(&cookie.encoded().to_string())?;

    let login_action = format!(
        "/oidc/{}/authorize/{transaction_sqid}/login",
        prepared.canonical_tenant_sqid,
    );
    let html = format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Sign in</title></head><body><main><h1>Sign in</h1><form method=\"post\" action=\"{login_action}\"><input type=\"hidden\" name=\"csrf\" value=\"{csrf}\"><p><label>Email or phone <input type=\"text\" name=\"identifier\" autocomplete=\"username\" required></label></p><p><label>Password <input type=\"password\" name=\"password\" autocomplete=\"current-password\" required></label></p><p><button type=\"submit\">Sign in</button></p></form><div id=\"oidc-authorization-context\" hidden data-transaction-sqid=\"{transaction_sqid}\" data-csrf=\"{csrf}\" data-revision=\"{}\"></div></main></body></html>",
        prepared.snapshot.revision,
    );
    let mut response = Html(html).into_response();
    response.headers_mut().insert(SET_COOKIE, cookie);
    response.headers_mut().insert(
        CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(AUTHORIZATION_FORM_CONTENT_SECURITY_POLICY_VALUE),
    );
    apply_common_headers(&mut response);
    Ok(response)
}

pub(super) fn local_response(status: StatusCode, message: &'static str) -> Response {
    let mut response = (status, message).into_response();
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    apply_common_headers(&mut response);
    response
}

pub(super) fn apply_common_headers(response: &mut Response) {
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static(CACHE_CONTROL_VALUE));
    response
        .headers_mut()
        .insert(PRAGMA, HeaderValue::from_static(PRAGMA_VALUE));
    response.headers_mut().insert(
        REFERRER_POLICY,
        HeaderValue::from_static(REFERRER_POLICY_VALUE),
    );
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
}
