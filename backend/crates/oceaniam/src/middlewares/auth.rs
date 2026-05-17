use std::convert::Infallible;

use crate::error::Error;
use crate::state::revoked::RevokedJwt;
use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, StatusCode, header, request::Parts},
};
use oceaniam_auth::{
    Algorithm, Header, TokenData, Validation, decode, decode_header,
    jwks::JwkSet,
    jwt::{Claim, SystemClaim},
};
use oceaniam_vo::sqid::Sqid;
use serde::{Serialize, de::DeserializeOwned};
use tap::Tap;
use tracing::error;
use uuid::Uuid;

use crate::state::AppState;
use crate::state::applications::UserIdentifier;

#[derive(Debug, Clone)]
pub struct PlatformAuthGuard {
    pub token: TokenData<SystemClaim>,
}

pub async fn validate<C>(
    header: &Header,
    jwks: JwkSet,
    token: String,
    validation: &Validation,
) -> Result<TokenData<C>, Error>
where
    C: Serialize + DeserializeOwned,
{
    // NOTE: We have already proved that `kid` must exist.
    let kid = header.kid.as_ref().unwrap();
    let span = tracing::debug_span!("jwt.validate", kid = %kid);
    let _guard = span.enter();

    let key = jwks.decoding_key_for_kid(kid).inspect_err(|e| {
        error!(
            %kid,
            error = %e,
            "failed to create decoding key from jwk"
        )
    })?;

    Ok(decode(token, &key, validation)
        .inspect_err(|e| error!(kid = %kid, error = %e, "failed to decode token"))?)
}

/// Extract and decode a Bearer token from the request's `Authorization` header.
///
/// Returns `(raw_token, decoded_header)` on success.  Fails with
/// [`NON_AUTHORITATIVE_INFORMATION`] when the header is absent, or with
/// [`BAD_REQUEST`] when the scheme is not `Bearer`, the JWT header is
/// malformed, or the required `kid` field is missing.
fn extract_bearer_token(parts: &Parts) -> Result<(String, Header), StatusCode> {
    let auth_header = parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let Some(token) = auth_header else {
        return Err(StatusCode::NON_AUTHORITATIVE_INFORMATION);
    };

    if !token.starts_with("Bearer ") {
        return Err(StatusCode::BAD_REQUEST);
    }

    let token = token.trim().replace("Bearer ", "");

    let Ok(header) = decode_header(&token) else {
        return Err(StatusCode::BAD_REQUEST);
    };

    if header.kid.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok((token, header))
}

/// Verify that the given JTI has not been revoked.
///
/// Returns `Ok(())` when the JTI is absent from the revocation store.
/// Returns [`BAD_REQUEST`] when the JTI has been revoked or the
/// revocation check itself fails.
async fn check_jti_not_revoked(revoked_jwt: &RevokedJwt, jti: Uuid) -> Result<(), StatusCode> {
    if let Ok(true) = revoked_jwt
        .is_revoked(jti)
        .await
        .inspect_err(|e| error!(%jti, error = %e, "failed to check jwt revocation"))
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(())
}

impl FromRequestParts<AppState<'_>> for PlatformAuthGuard {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        AppState {
            platform_jwks: system_jwks,
            platform_jwt_validator: system_jwt_validator,
            revoked_jwt,
            ..
        }: &AppState<'_>,
    ) -> Result<Self, Self::Rejection> {
        let (token, header) = extract_bearer_token(parts)?;

        let Ok(token) =
            validate::<SystemClaim>(&header, system_jwks.jwks(), token, system_jwt_validator).await
        else {
            return Err(StatusCode::BAD_REQUEST);
        };

        check_jti_not_revoked(revoked_jwt, token.claims.jti).await?;

        Ok(Self { token })
    }
}

/// Guard that validates a JWT against the **application-specific** JWKS and
/// token configuration (`issuer`, `audience`) extracted from the request path.
///
/// Unlike [`PlatformAuthGuard`] (which always uses the system-level JWKS), this
/// guard loads the key set and validation parameters belonging to the
/// application identified by the `application_id` path segment, so that
/// application tokens are verified with the same keys and settings that were
/// used to sign them.
///
/// # Panics
///
/// The guard expects the request URI to follow the convention
/// `/tenants/{id}/applications/{id}/…`.  If the path does not match this
/// pattern the guard will always fail with `400 Bad Request`.
#[derive(Debug, Clone)]
pub struct ApplicationAuthGuard {
    pub token: TokenData<Claim>,
}

impl FromRequestParts<AppState<'_>> for ApplicationAuthGuard {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        AppState {
            keyboxes,
            applications,
            revoked_jwt,
            ..
        }: &AppState<'_>,
    ) -> Result<Self, Self::Rejection> {
        let (token, header) = extract_bearer_token(parts)?;

        let path_segments: Vec<&str> = parts.uri.path().split('/').collect();
        let application_id = path_segments
            .iter()
            .position(|&segment| segment == "applications")
            .and_then(|idx| path_segments.get(idx + 1))
            .and_then(|id| id.parse::<Sqid>().ok())
            .and_then(|id| Uuid::try_from(id).ok())
            .ok_or(StatusCode::BAD_REQUEST)?;

        let jwks = keyboxes
            .get_jwks(application_id)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        let config = applications
            .get_configuration(application_id)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        let validation = Validation::default()
            .tap_mut(|it| it.aud = Some(config.auth.token.audience.into_iter().collect()))
            .tap_mut(|it| it.iss = Some([config.auth.token.issuer].into_iter().collect()))
            .tap_mut(|it| {
                it.algorithms = vec![
                    Algorithm::PS256,
                    Algorithm::PS384,
                    Algorithm::PS512,
                    Algorithm::RS256,
                    Algorithm::RS384,
                    Algorithm::RS512,
                ]
            });

        let Ok(token) = validate::<Claim>(&header, jwks, token, &validation).await else {
            return Err(StatusCode::BAD_REQUEST);
        };

        check_jti_not_revoked(revoked_jwt, token.claims.jti).await?;

        // NOTE: Defence-in-depth — confirm the subject belongs to the application.
        let _ = applications
            .find_user_by(application_id, UserIdentifier::Id(token.claims.sub))
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        Ok(Self { token })
    }
}

/// How an authentication token should be delivered back to the client.
///
/// This is typically chosen by the endpoint that issues or refreshes tokens.
///
/// The dispatch method can be controlled by the request header
/// `X-OceanIAM-Token-Dispatch`.
///
/// - Accepted values: `cookie`, `json`, `both` (case-insensitive; whitespace is ignored)
/// - Default: missing/invalid header falls back to `both`
///
/// Common use-cases:
/// - Browser apps often prefer `Cookie` so the token is automatically attached
///   to subsequent requests (and can be protected with `HttpOnly`, `Secure`,
///   `SameSite`, etc.).
/// - API clients (mobile/desktop/CLI/SDK) often prefer `Json` so they can
///   manage the token explicitly (e.g., attach it as an `Authorization: Bearer`
///   header).
/// - During migrations or when you need to support both kinds of clients,
///   `Both` can be used to send the token in both places.
#[derive(Debug, Clone)]
pub enum TokenDispatchMethodGuard {
    /// Deliver the token via an HTTP cookie (e.g., `Set-Cookie`).
    ///
    /// Useful when:
    /// - The client is a browser and you want session-like behavior.
    /// - You want to use cookie attributes (`HttpOnly`, `Secure`, `SameSite`)
    ///   to improve safety ergonomics.
    Cookie,

    /// Deliver the token via JSON response body.
    ///
    /// Useful when:
    /// - The client is not a browser (mobile, desktop, CLI, server-to-server).
    /// - The client wants to decide how/where to persist the token.
    Json,

    /// Deliver the token both via cookie and JSON.
    ///
    /// Useful when:
    /// - You have mixed clients and want a single response format.
    /// - You are migrating between `Cookie` and `Json` based auth flows.
    Both,
}

const TOKEN_DISPATCH_METHOD_HEADER: &str = "X-OceanIAM-Token-Dispatch";

impl TokenDispatchMethodGuard {
    fn from_headers(headers: &HeaderMap) -> Self {
        let Some(method) = headers.get(TOKEN_DISPATCH_METHOD_HEADER) else {
            return Self::Both;
        };

        let Ok(method) = method.to_str() else {
            return Self::Both;
        };

        let normalized = method
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect::<String>()
            .to_ascii_lowercase();

        match normalized.as_str() {
            "cookie" => Self::Cookie,
            "json" => Self::Json,
            "both" => Self::Both,
            _ => Self::Both,
        }
    }
}

impl FromRequestParts<AppState<'_>> for TokenDispatchMethodGuard {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        _: &AppState<'_>,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self::from_headers(&parts.headers))
    }
}

#[cfg(test)]
mod tests {
    use super::{TOKEN_DISPATCH_METHOD_HEADER, TokenDispatchMethodGuard};

    use axum::http::{HeaderMap, HeaderValue};

    // NOTE: AI-generated test
    #[test]
    fn default_to_both_when_header_missing() {
        let headers = HeaderMap::new();
        assert!(matches!(
            TokenDispatchMethodGuard::from_headers(&headers),
            TokenDispatchMethodGuard::Both
        ));
    }

    // NOTE: AI-generated test
    #[test]
    fn parse_cookie_json_both_case_insensitive_and_whitespace_tolerant() {
        let mut headers = HeaderMap::new();

        headers.insert(
            TOKEN_DISPATCH_METHOD_HEADER,
            HeaderValue::from_static("cookie"),
        );
        assert!(matches!(
            TokenDispatchMethodGuard::from_headers(&headers),
            TokenDispatchMethodGuard::Cookie
        ));

        headers.insert(
            TOKEN_DISPATCH_METHOD_HEADER,
            HeaderValue::from_static("  JSoN  "),
        );
        assert!(matches!(
            TokenDispatchMethodGuard::from_headers(&headers),
            TokenDispatchMethodGuard::Json
        ));

        headers.insert(
            TOKEN_DISPATCH_METHOD_HEADER,
            HeaderValue::from_static(" b o t h "),
        );
        assert!(matches!(
            TokenDispatchMethodGuard::from_headers(&headers),
            TokenDispatchMethodGuard::Both
        ));
    }

    // NOTE: AI-generated test
    #[test]
    fn fallback_to_both_when_header_invalid_or_non_utf8() {
        let mut headers = HeaderMap::new();

        headers.insert(
            TOKEN_DISPATCH_METHOD_HEADER,
            HeaderValue::from_static("unknown"),
        );
        assert!(matches!(
            TokenDispatchMethodGuard::from_headers(&headers),
            TokenDispatchMethodGuard::Both
        ));

        let non_utf8 = HeaderValue::from_bytes(&[0xff, 0xfe]).unwrap();
        headers.insert(TOKEN_DISPATCH_METHOD_HEADER, non_utf8);
        assert!(matches!(
            TokenDispatchMethodGuard::from_headers(&headers),
            TokenDispatchMethodGuard::Both
        ));
    }
}
