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
pub(crate) fn extract_bearer_token(parts: &Parts) -> Result<(String, Header), StatusCode> {
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
