use axum::http::{StatusCode, header, request::Parts};
use oceaniam_auth::{Header, TokenData, Validation, decode, decode_header, jwks::JwkSet};
use oceaniam_common::run_cpu_bound;
use serde::{Serialize, de::DeserializeOwned};
use tracing::{error, field};
use uuid::Uuid;

use crate::error::Error;
use crate::state::revoked::RevokedJwt;

/// Verifies a JWT against the tenant JWKS.
///
/// The RSA verification is CPU-bound, so the whole section (decoding-key construction plus
/// the jsonwebtoken `decode`) is dispatched through [`run_cpu_bound`], mirroring the signing
/// path in `state::keybox`: a queue span records semaphore wait, the inner span carries
/// `otel.status_code`/`otel.status_description` on error, and neither the token nor any key
/// material is recorded.
pub async fn validate<C>(
    header: &Header,
    jwks: JwkSet,
    token: String,
    validation: &Validation,
) -> Result<TokenData<C>, Error>
where
    C: Serialize + DeserializeOwned + Send + 'static,
{
    // NOTE: We have already proved that `kid` must exist.
    let kid = header.kid.as_ref().unwrap().to_owned();
    let validation = validation.clone();
    let queue_span = tracing::info_span!(
        "auth.jwt.validate.queue",
        otel.kind = "internal",
        cpu.operation = "rsa.validate_jwt",
        key.id = %kid
    );

    run_cpu_bound(queue_span, move |parent| {
        let span = tracing::info_span!(
            parent: &parent,
            "auth.jwt.validate",
            otel.kind = "internal",
            otel.status_code = field::Empty,
            otel.status_description = field::Empty,
            cpu.operation = "rsa.validate_jwt",
            key.id = %kid,
        );
        let result = span.in_scope(|| {
            let key = jwks.decoding_key_for_kid(&kid).inspect_err(|e| {
                error!(
                    %kid,
                    error = %e,
                    "failed to create decoding key from jwk"
                )
            })?;

            decode(token, &key, &validation)
                .inspect_err(|e| error!(%kid, error = %e, "failed to decode token"))
                .map_err(Into::into)
        });

        if result.is_err() {
            span.record("otel.status_code", "ERROR");
            span.record("otel.status_description", "RSA JWT validation failed");
        }

        result
    })
    .await
    .map_err(|source| Error::Internal {
        msg: format!("CPU-bound RSA JWT validation task failed: {source}"),
        location: snafu::location!(),
    })?
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
    use std::io::{self, Write};
    use std::sync::{Arc, Mutex};

    use axum::http::{Request, StatusCode, header};
    use oceaniam_auth::{
        Algorithm, Header, TokenData, Validation, decode_header,
        jwks::JwkSet,
        jwt::{ClaimHelper, JwtCodec, SystemClaim},
    };
    use oceaniam_keybox::{KeyAlg, RsaKey, TryIntoJwk};
    use tap::Tap;
    use tracing::Instrument as _;
    use tracing_subscriber::fmt::format::FmtSpan;
    use uuid::Uuid;

    use super::{extract_bearer_token, validate};

    #[derive(Clone)]
    struct Buffer(Arc<Mutex<Vec<u8>>>);

    impl Write for Buffer {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0
                .lock()
                .expect("trace buffer lock")
                .extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    // NOTE: AI-generated test
    #[tokio::test]
    async fn jwt_validation_spans_cross_blocking_dispatch_without_token_or_key_material() {
        let key_id = Uuid::now_v7();
        let key = RsaKey::with_bit_size(
            key_id,
            KeyAlg::try_from(Algorithm::RS256).expect("supported algorithm"),
            2048,
        )
        .expect("generate test RSA key");
        let subject = Uuid::now_v7();
        let token = key
            .clone()
            .encode(
                Header::new(Algorithm::RS256).tap_mut(|it| it.kid = Some(key_id.to_string())),
                SystemClaim::new(subject, 60, None, None),
            )
            .expect("sign test JWT");
        let jwks = JwkSet {
            keys: std::iter::once(key.try_into_jwk().expect("public JWK")).collect(),
        };
        let header = decode_header(&token).expect("decode test JWT header");
        let validation = Validation::new(Algorithm::RS256);

        let bytes = Arc::new(Mutex::new(Vec::new()));
        let writer = bytes.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(move || Buffer(writer.clone()))
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .finish();
        let dispatch = tracing::Dispatch::new(subscriber);
        let _default = tracing::dispatcher::set_default(&dispatch);

        let decoded: TokenData<SystemClaim> =
            async { validate(&header, jwks, token.clone(), &validation).await }
                .instrument(tracing::info_span!("test.request"))
                .await
                .expect("validate test JWT");
        assert_eq!(decoded.claims.sub, subject);

        let output = String::from_utf8(bytes.lock().expect("trace buffer lock").clone())
            .expect("utf8 trace output");
        assert!(output.contains("auth.jwt.validate.queue"));
        assert!(output.contains("auth.jwt.validate"));
        assert!(output.contains("auth.jwk.decoding_key"));
        assert!(output.contains("test.request"));
        assert!(!output.contains(&token));
        assert!(
            !output.contains("PRIVATE KEY"),
            "key material must never appear in trace output"
        );
    }

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
