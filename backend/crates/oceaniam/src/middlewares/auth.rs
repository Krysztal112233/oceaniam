use std::convert::Infallible;

use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, StatusCode, header, request::Parts},
};
use jsonwebtoken::{DecodingKey, Header, TokenData, Validation, decode, decode_header};
use oceaniam_common::{error::Error, jwks::JwkSet, jwt::ClaimHelper};
use serde::{Serialize, de::DeserializeOwned};
use tracing::error;

use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct RequireAuth<S> {
    pub token: TokenData<S>,
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

    let key = {
        let tmp = jsonwebtoken::jwk::JwkSet::from(jwks);
        let Some(jwk) = tmp.find(kid) else {
            return Err(Error::with_code(
                StatusCode::BAD_REQUEST,
                format!("cannot find jwk for kid `{kid}`."),
            ));
        };

        DecodingKey::from_jwk(jwk).inspect_err(|e| {
            error!(
                kid = %kid,
                error = %e,
                "failed to create decoding key from jwk"
            )
        })?
    };

    Ok(decode(token, &key, validation)
        .inspect_err(|e| error!(kid = %kid, error = %e, "failed to decode token"))?)
}

impl<C> FromRequestParts<AppState<'_>> for RequireAuth<C>
where
    C: Serialize + DeserializeOwned + ClaimHelper + Send,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        AppState {
            system_jwks,
            system_jwt_validator: jwt_validator,
            revoked_jwt,
            ..
        }: &AppState<'_>,
    ) -> Result<Self, Self::Rejection> {
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

        // NOTE: By design, token's header must have `kid` field.
        if header.kid.is_none() {
            return Err(StatusCode::BAD_REQUEST);
        }

        let Ok(token) = validate::<C>(&header, system_jwks.jwks(), token, jwt_validator).await
        else {
            return Err(StatusCode::BAD_REQUEST);
        };

        let jti = token.claims.jti();
        if let Ok(true) = revoked_jwt.is_revoked(jti).await.inspect_err(|e| {
            error!(
                jti = %jti,
                error = %e,
                "failed to check jwt revocation or database failed"
            )
        }) {
            return Err(StatusCode::BAD_REQUEST);
        }

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
pub enum TokenDispatchMethod {
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

impl TokenDispatchMethod {
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

impl FromRequestParts<AppState<'_>> for TokenDispatchMethod {
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
    use super::{TOKEN_DISPATCH_METHOD_HEADER, TokenDispatchMethod};

    use axum::http::{HeaderMap, HeaderValue};

    // NOTE: AI-generated test
    #[test]
    fn default_to_both_when_header_missing() {
        let headers = HeaderMap::new();
        assert!(matches!(
            TokenDispatchMethod::from_headers(&headers),
            TokenDispatchMethod::Both
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
            TokenDispatchMethod::from_headers(&headers),
            TokenDispatchMethod::Cookie
        ));

        headers.insert(
            TOKEN_DISPATCH_METHOD_HEADER,
            HeaderValue::from_static("  JSoN  "),
        );
        assert!(matches!(
            TokenDispatchMethod::from_headers(&headers),
            TokenDispatchMethod::Json
        ));

        headers.insert(
            TOKEN_DISPATCH_METHOD_HEADER,
            HeaderValue::from_static(" b o t h "),
        );
        assert!(matches!(
            TokenDispatchMethod::from_headers(&headers),
            TokenDispatchMethod::Both
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
            TokenDispatchMethod::from_headers(&headers),
            TokenDispatchMethod::Both
        ));

        let non_utf8 = HeaderValue::from_bytes(&[0xff, 0xfe]).unwrap();
        headers.insert(TOKEN_DISPATCH_METHOD_HEADER, non_utf8);
        assert!(matches!(
            TokenDispatchMethod::from_headers(&headers),
            TokenDispatchMethod::Both
        ));
    }
}
