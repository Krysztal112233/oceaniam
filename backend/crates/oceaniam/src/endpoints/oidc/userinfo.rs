//! OIDC UserInfo endpoint: OIDC Core §5.3 over the RFC 6750 bearer transport.
//!
//! GET and POST answer identically (OIDC Core §5.3.1) and accept the access token only
//! through the RFC 6750 §2.1 `Authorization: Bearer` header; the query string and the POST
//! body are never consulted for the token. Verification is ordered so an absent tenant never
//! touches the keybox, and every presented-token failure — non-Bearer scheme, malformed JWT
//! header, unknown key, bad signature, expiration, issuer mismatch, unregistered audience,
//! or deleted subject — collapses into one byte-identical 401 with an RFC 6750 §3.1
//! `invalid_token` challenge, so the failure cause cannot be probed online. The only claim
//! this slice exposes is `sub` (scope support is `openid` only; profile and email claims
//! await the scope slice). Access tokens carry no revocation mechanism yet, so no `jti`
//! lookup occurs, and this read-only endpoint writes no audit event. The presented token is
//! never logged or echoed.

use std::convert::Infallible;

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use oceaniam_auth::{Algorithm, Validation, decode_header, jwt::Claim};
use oceaniam_common::sqid::Sqid;
use oceaniam_database::{
    helper::{oidc_clients::OidcClientsHelper, tenants::TenantsHelper, users::UserHelper},
    model::prelude::{OidcClients, Tenants, Users},
};
use serde::Serialize;
use tap::Tap;
use tracing::error;
use utoipa::ToSchema;
use utoipa_axum::{router::UtoipaMethodRouter, routes};
use uuid::Uuid;

use crate::{state::AppState, util::jwt};

use super::{
    authorize::{PreviewEnabled, apply_common_headers, local_response, tenant_issuer},
    login::is_unavailable,
};

/// RFC 6750 §3.1 error code for a presented token that failed any verification step.
const INVALID_TOKEN: &str = "invalid_token";

/// OIDC Core §5.3.2 successful UserInfo response. `sub` (the stable external
/// `users.oidc_sub`) is the only claim this slice exposes: scope support is `openid` only.
#[derive(Serialize, ToSchema)]
struct OidcUserInfoResponse {
    sub: Uuid,
}

/// Default-disabled OIDC UserInfo endpoint over GET.
#[utoipa::path(
    get,
    path = "/oidc/{tenant_sqid}/userinfo",
    tag = "Oidc",
    params(
        ("tenant_sqid" = String, Path, description = "Tenant Sqid"),
    ),
    responses(
        (status = 200, description = "OIDC Core 5.3.2 UserInfo response; only `sub` is exposed this slice", body = OidcUserInfoResponse),
        (status = 400, description = "Malformed tenant identifier; the RFC 6750 3.1 challenge carries `invalid_request` without a realm", body = String,
            headers(("WWW-Authenticate" = String, description = "RFC 6750 3.1 challenge: `Bearer error=\"invalid_request\"`, no realm"))),
        (status = 401, description = "RFC 6750 3.1 bearer challenge: no token presented (no error code) or a presented token failed verification (`invalid_token`)", body = String,
            headers(("WWW-Authenticate" = String, description = "RFC 6750 3.1 challenge with the tenant issuer realm"))),
        (status = 404, description = "Preview disabled or tenant unavailable", body = String),
        (status = 405, description = "Only GET and POST are supported (empty body; HEAD follows GET)"),
        (status = 500, description = "Internal server error", body = String),
    ),
)]
#[tracing::instrument(level = "info", name = "oidc.userinfo.get", skip_all)]
async fn get_oidc_userinfo(
    _enabled: PreviewEnabled,
    Path(tenant_sqid): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    handle_userinfo(&state, &tenant_sqid, &headers).await
}

/// Default-disabled OIDC UserInfo endpoint over POST. The request body is ignored entirely:
/// the token transport is the RFC 6750 §2.1 header only.
#[utoipa::path(
    post,
    path = "/oidc/{tenant_sqid}/userinfo",
    tag = "Oidc",
    params(
        ("tenant_sqid" = String, Path, description = "Tenant Sqid"),
    ),
    responses(
        (status = 200, description = "OIDC Core 5.3.2 UserInfo response; only `sub` is exposed this slice", body = OidcUserInfoResponse),
        (status = 400, description = "Malformed tenant identifier; the RFC 6750 3.1 challenge carries `invalid_request` without a realm", body = String,
            headers(("WWW-Authenticate" = String, description = "RFC 6750 3.1 challenge: `Bearer error=\"invalid_request\"`, no realm"))),
        (status = 401, description = "RFC 6750 3.1 bearer challenge: no token presented (no error code) or a presented token failed verification (`invalid_token`)", body = String,
            headers(("WWW-Authenticate" = String, description = "RFC 6750 3.1 challenge with the tenant issuer realm"))),
        (status = 404, description = "Preview disabled or tenant unavailable", body = String),
        (status = 405, description = "Only GET and POST are supported (empty body; HEAD follows GET)"),
        (status = 500, description = "Internal server error", body = String),
    ),
)]
#[tracing::instrument(level = "info", name = "oidc.userinfo.post", skip_all)]
async fn post_oidc_userinfo(
    _enabled: PreviewEnabled,
    Path(tenant_sqid): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    handle_userinfo(&state, &tenant_sqid, &headers).await
}

async fn handle_userinfo(state: &AppState, tenant_sqid: &str, headers: &HeaderMap) -> Response {
    // A malformed tenant identifier has no derivable issuer realm, so the RFC 6750 §3.1
    // challenge carries `invalid_request` without a realm.
    let Some(tenant_id) = tenant_sqid
        .parse::<Sqid>()
        .ok()
        .and_then(|sqid| Uuid::try_from(sqid).ok())
    else {
        return challenge_response(
            StatusCode::BAD_REQUEST,
            "Bearer error=\"invalid_request\"",
            "invalid tenant identifier",
        );
    };

    // System-protected tenant resolution before any token work: unknown and system tenants
    // answer 404 (namespace parity with the JWKS route) and never touch the keybox.
    if let Err(error) = Tenants::get_tenant(tenant_id, &state.database).await {
        if is_unavailable(&error) {
            return local_response(StatusCode::NOT_FOUND, "not found");
        }
        error!(?error, "failed to resolve the OIDC UserInfo tenant");
        return local_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error");
    }

    let issuer = tenant_issuer(&state.public_base_url, &Sqid::from(tenant_id));

    // RFC 6750 §2.1 header transport only; the query string and the POST body are never
    // consulted for the token.
    let Some(authorization) = headers.get(header::AUTHORIZATION) else {
        // RFC 6750 §3.1: no token was presented, so the challenge carries no error code.
        return challenge_response(
            StatusCode::UNAUTHORIZED,
            &format!("Bearer realm=\"{issuer}\""),
            "missing bearer token",
        );
    };
    // A present but non-UTF-8 header still presented credentials, so it takes the
    // `invalid_token` path rather than the missing-token challenge.
    let Ok(authorization) = authorization.to_str() else {
        return invalid_token_response(&issuer);
    };
    // RFC 7235: the scheme is matched case-insensitively; the token is the verbatim
    // remainder after the first ASCII space.
    let Some((scheme, token)) = authorization.split_once(' ') else {
        return invalid_token_response(&issuer);
    };
    if !scheme.eq_ignore_ascii_case("bearer") {
        return invalid_token_response(&issuer);
    }
    let Ok(header) = decode_header(token) else {
        return invalid_token_response(&issuer);
    };
    if header.kid.is_none() {
        return invalid_token_response(&issuer);
    }

    let jwks = match state.keyboxes.get_jwks(tenant_id).await {
        Ok(jwks) => jwks,
        Err(error) => {
            error!(?error, "failed to load the OIDC UserInfo tenant JWKS");
            return local_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error");
        }
    };

    // The audience is validated against the client registry below, not here: jsonwebtoken
    // rejects any present `aud` when the Validation carries none, so `validate_aud` is off.
    // jsonwebtoken only validates `iss` when the token carries one, so `iss` joins the
    // required spec claims (default: only `exp`) and an issuer-less token is rejected.
    let validation = Validation::default()
        .tap_mut(|it| it.validate_aud = false)
        .tap_mut(|it| {
            it.required_spec_claims.insert("iss".to_owned());
        })
        .tap_mut(|it| it.iss = Some([issuer.clone()].into_iter().collect()))
        .tap_mut(|it| {
            it.algorithms = vec![
                Algorithm::PS256,
                Algorithm::PS384,
                Algorithm::PS512,
                Algorithm::RS256,
                Algorithm::RS384,
                Algorithm::RS512,
            ];
        });

    let Ok(token_data) = jwt::validate::<Claim>(&header, jwks, token.to_owned(), &validation).await
    else {
        return invalid_token_response(&issuer);
    };
    let claim = token_data.claims;

    // At least one audience must resolve to a registered client whose application belongs to
    // the path tenant; the resolved client scopes the subject lookup below.
    let Some(audiences) = claim.aud.filter(|audiences| !audiences.is_empty()) else {
        return invalid_token_response(&issuer);
    };
    let mut client = None;
    for audience in &audiences {
        match OidcClients::get_client_of_tenant(tenant_id, audience, &state.database).await {
            Ok(resolved) => {
                client = Some(resolved);
                break;
            }
            Err(error) if is_unavailable(&error) => continue,
            Err(error) => {
                error!(
                    ?error,
                    "failed to resolve the OIDC UserInfo audience client"
                );
                return local_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error");
            }
        }
    }
    let Some(client) = client else {
        return invalid_token_response(&issuer);
    };

    // Fail closed for deleted subjects: the access token outlives the user row.
    let subject = match Users::resolve_oidc_subject(
        client.application_id,
        claim.sub,
        &state.database,
    )
    .await
    {
        Ok(subject) => subject,
        Err(error) if is_unavailable(&error) => return invalid_token_response(&issuer),
        Err(error) => {
            error!(?error, "failed to resolve the OIDC UserInfo subject");
            return local_response(StatusCode::INTERNAL_SERVER_ERROR, "internal server error");
        }
    };

    // No jti revocation check: OIDC access tokens have no revocation mechanism yet (deferred
    // to the refresh slice).
    let mut response = Json(OidcUserInfoResponse {
        sub: subject.oidc_sub,
    })
    .into_response();
    apply_common_headers(&mut response);
    response
}

/// RFC 6750 §3.1: a presented token that failed any verification step collapses into one
/// byte-identical 401 (same status, headers, and body), so the failure cause cannot be probed
/// online. The presented token is never echoed.
fn invalid_token_response(issuer: &str) -> Response {
    challenge_response(
        StatusCode::UNAUTHORIZED,
        &format!("Bearer realm=\"{issuer}\", error=\"{INVALID_TOKEN}\""),
        "invalid access token",
    )
}

/// Family plain-text body plus the RFC 6750 §3.1 challenge. The issuer realm is a
/// config-derived URL, so the challenge value never contains a DQUOTE or backslash and needs
/// no escaping.
fn challenge_response(status: StatusCode, challenge: &str, message: &'static str) -> Response {
    let mut response = local_response(status, message);
    match HeaderValue::from_str(challenge) {
        Ok(value) => {
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, value);
        }
        Err(error) => {
            error!(
                ?error,
                "failed to build the WWW-Authenticate challenge header"
            );
        }
    }
    response
}

pub(super) fn routes() -> UtoipaMethodRouter<AppState, Infallible> {
    routes!(get_oidc_userinfo, post_oidc_userinfo)
}
