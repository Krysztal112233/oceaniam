use axum::http::{StatusCode, header, request::Parts};
use oceaniam_auth::{Header, TokenData, Validation, decode, decode_header, jwks::JwkSet};
use serde::{Serialize, de::DeserializeOwned};
use tracing::error;
use uuid::Uuid;

use crate::error::Error;
use crate::state::revoked::RevokedJwt;

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
    let span = tracing::info_span!(
        "auth.jwt.validate",
        otel.kind = "internal",
        key.id = %kid
    );
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
/// RFC 7235: the scheme is matched case-insensitively and the token is the verbatim
/// remainder after the first ASCII space.
///
/// Returns `(raw_token, decoded_header)` on success.  Fails with
/// [`NON_AUTHORITATIVE_INFORMATION`] when the header is absent, or with
/// [`BAD_REQUEST`] when the scheme is not `Bearer`, the JWT header is
/// malformed, or the required `kid` field is missing.
pub(crate) fn extract_bearer_token(parts: &Parts) -> Result<(String, Header), StatusCode> {
    let auth_header = parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let Some(auth_header) = auth_header else {
        return Err(StatusCode::NON_AUTHORITATIVE_INFORMATION);
    };

    let Some((scheme, token)) = auth_header.split_once(' ') else {
        return Err(StatusCode::BAD_REQUEST);
    };
    if !scheme.eq_ignore_ascii_case("bearer") {
        return Err(StatusCode::BAD_REQUEST);
    }

    let token = token.to_owned();

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
pub(crate) async fn check_jti_not_revoked(
    revoked_jwt: &RevokedJwt,
    jti: Uuid,
) -> Result<(), StatusCode> {
    if let Ok(true) = revoked_jwt
        .is_revoked(jti)
        .await
        .inspect_err(|e| error!(%jti, error = %e, "failed to check jwt revocation"))
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use axum::http::{Request, StatusCode, header};

    use super::extract_bearer_token;

    // NOTE: AI-generated test
    #[test]
    fn bearer_token_without_key_id_is_rejected_at_extraction() {
        let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.e30.signature";
        let (parts, _) = Request::builder()
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .body(())
            .expect("construct bearer request")
            .into_parts();

        assert!(matches!(
            extract_bearer_token(&parts),
            Err(StatusCode::BAD_REQUEST)
        ));
    }

    // NOTE: AI-generated test
    #[test]
    fn bearer_scheme_is_matched_case_insensitively() {
        // Header: {"typ":"JWT","alg":"RS256","kid":"abc"}
        let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsImtpZCI6ImFiYyJ9.e30.signature";
        for scheme in ["Bearer", "bearer", "BEARER", "BeArEr"] {
            let (parts, _) = Request::builder()
                .header(header::AUTHORIZATION, format!("{scheme} {token}"))
                .body(())
                .expect("construct bearer request")
                .into_parts();

            let (extracted, header) = extract_bearer_token(&parts)
                .unwrap_or_else(|status| panic!("scheme {scheme} should be accepted: {status}"));
            assert_eq!(extracted, token, "the token is the verbatim remainder");
            assert_eq!(header.kid.as_deref(), Some("abc"));
        }
    }

    // NOTE: AI-generated test
    #[test]
    fn malformed_authorization_headers_keep_their_error_split() {
        let (parts, _) = Request::builder()
            .body(())
            .expect("construct request")
            .into_parts();
        assert!(matches!(
            extract_bearer_token(&parts),
            Err(StatusCode::NON_AUTHORITATIVE_INFORMATION)
        ));

        for (label, value) in [
            ("non-bearer scheme", "Basic dXNlcjpwYXNz"),
            ("scheme without a token", "Bearer"),
            ("scheme with an empty token", "Bearer "),
            ("malformed jwt", "Bearer not-a-jwt"),
            ("missing scheme", "not-a-jwt"),
        ] {
            let (parts, _) = Request::builder()
                .header(header::AUTHORIZATION, value)
                .body(())
                .expect("construct request")
                .into_parts();
            assert!(
                matches!(extract_bearer_token(&parts), Err(StatusCode::BAD_REQUEST)),
                "{label}"
            );
        }
    }
}
