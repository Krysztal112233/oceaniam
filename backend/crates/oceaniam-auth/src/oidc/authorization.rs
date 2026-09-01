//! Request-local validation for the future OIDC Authorization Endpoint.
//!
//! This module deliberately does not validate client registration, tenant ownership, or exact
//! redirect URI registration, and it does not decide whether an error may be returned by redirect.
//! A future endpoint/parser must also define handling for duplicate, unknown, and unsupported
//! extension parameters before this type becomes reachable over HTTP.

use serde::Deserialize;
use snafu::Snafu;
use url::Url;

const RESPONSE_TYPE_CODE: &str = "code";
const SCOPE_OPENID: &str = "openid";
const CODE_CHALLENGE_METHOD_S256: &str = "S256";
const S256_CODE_CHALLENGE_LENGTH: usize = 43;

/// Deserializable Authorization Endpoint parameters before protocol validation.
///
/// Fields are optional so a future query extractor can report deterministic validation errors for
/// missing parameters instead of failing solely during Serde deserialization. This is not a full
/// Authorization Endpoint parser; unsupported extension-parameter handling remains out of scope.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RawAuthorizationRequest {
    response_type: Option<String>,
    client_id: Option<String>,
    redirect_uri: Option<String>,
    scope: Option<String>,
    state: Option<String>,
    nonce: Option<String>,
    code_challenge: Option<String>,
    code_challenge_method: Option<String>,
}

/// Authorization request that satisfies OceanIAM's request-local v1 profile.
///
/// OceanIAM's v1 profile is deliberately stricter than the base protocol: it supports only the
/// authorization-code response type and `openid` scope, and requires both `state` and explicit
/// PKCE S256. Construction does not prove that the client or redirect URI is registered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationRequest {
    client_id: String,
    redirect_uri: String,
    state: String,
    nonce: Option<String>,
    code_challenge: String,
}

impl AuthorizationRequest {
    pub const fn response_type(&self) -> &'static str {
        RESPONSE_TYPE_CODE
    }

    /// Returns the public client identifier; it is never a credential or secret.
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    /// Returns the original, non-normalized redirect URI for future exact registration matching.
    pub fn redirect_uri(&self) -> &str {
        &self.redirect_uri
    }

    pub const fn scope(&self) -> &'static str {
        SCOPE_OPENID
    }

    /// Returns the exact state value supplied by the client.
    pub fn state(&self) -> &str {
        &self.state
    }

    pub fn nonce(&self) -> Option<&str> {
        self.nonce.as_deref()
    }

    pub fn code_challenge(&self) -> &str {
        &self.code_challenge
    }

    pub const fn code_challenge_method(&self) -> &'static str {
        CODE_CHALLENGE_METHOD_S256
    }
}

/// Deterministic request-local validation failures, without HTTP or redirect semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Snafu)]
#[non_exhaustive]
pub enum AuthorizationRequestError {
    #[snafu(display("response_type is required"))]
    MissingResponseType,

    #[snafu(display("response_type is not supported"))]
    UnsupportedResponseType,

    #[snafu(display("client_id is required"))]
    MissingClientId,

    #[snafu(display("client_id must not be empty"))]
    EmptyClientId,

    #[snafu(display("redirect_uri is required"))]
    MissingRedirectUri,

    #[snafu(display("redirect_uri must be an absolute URI"))]
    InvalidRedirectUri,

    #[snafu(display("redirect_uri must not include a fragment"))]
    RedirectUriFragmentNotAllowed,

    #[snafu(display("scope is required"))]
    MissingScope,

    #[snafu(display("scope is not supported"))]
    UnsupportedScope,

    #[snafu(display("state is required by OceanIAM policy"))]
    MissingState,

    #[snafu(display("state must not be empty"))]
    EmptyState,

    #[snafu(display("code_challenge_method is required by OceanIAM policy"))]
    MissingCodeChallengeMethod,

    #[snafu(display("code_challenge_method is not supported"))]
    UnsupportedCodeChallengeMethod,

    #[snafu(display("code_challenge is required by OceanIAM policy"))]
    MissingCodeChallenge,

    #[snafu(display("code_challenge is not a canonical S256 challenge"))]
    InvalidCodeChallenge,

    #[snafu(display("nonce must not be empty when present"))]
    EmptyNonce,
}

impl TryFrom<RawAuthorizationRequest> for AuthorizationRequest {
    type Error = AuthorizationRequestError;

    fn try_from(raw: RawAuthorizationRequest) -> Result<Self, Self::Error> {
        let RawAuthorizationRequest {
            response_type,
            client_id,
            redirect_uri,
            scope,
            state,
            nonce,
            code_challenge,
            code_challenge_method,
        } = raw;

        let response_type = response_type.ok_or(AuthorizationRequestError::MissingResponseType)?;
        if response_type != RESPONSE_TYPE_CODE {
            return Err(AuthorizationRequestError::UnsupportedResponseType);
        }

        let client_id = client_id.ok_or(AuthorizationRequestError::MissingClientId)?;
        if client_id.is_empty() {
            return Err(AuthorizationRequestError::EmptyClientId);
        }

        let redirect_uri = redirect_uri.ok_or(AuthorizationRequestError::MissingRedirectUri)?;
        if redirect_uri
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte == b' ')
        {
            return Err(AuthorizationRequestError::InvalidRedirectUri);
        }
        let parsed_redirect_uri =
            Url::parse(&redirect_uri).map_err(|_| AuthorizationRequestError::InvalidRedirectUri)?;
        if parsed_redirect_uri.fragment().is_some() {
            return Err(AuthorizationRequestError::RedirectUriFragmentNotAllowed);
        }

        let scope = scope.ok_or(AuthorizationRequestError::MissingScope)?;
        if scope != SCOPE_OPENID {
            return Err(AuthorizationRequestError::UnsupportedScope);
        }

        let state = state.ok_or(AuthorizationRequestError::MissingState)?;
        if state.is_empty() {
            return Err(AuthorizationRequestError::EmptyState);
        }

        let code_challenge_method =
            code_challenge_method.ok_or(AuthorizationRequestError::MissingCodeChallengeMethod)?;
        if code_challenge_method != CODE_CHALLENGE_METHOD_S256 {
            return Err(AuthorizationRequestError::UnsupportedCodeChallengeMethod);
        }

        let code_challenge =
            code_challenge.ok_or(AuthorizationRequestError::MissingCodeChallenge)?;
        if !is_canonical_s256_code_challenge(&code_challenge) {
            return Err(AuthorizationRequestError::InvalidCodeChallenge);
        }

        if nonce.as_deref().is_some_and(str::is_empty) {
            return Err(AuthorizationRequestError::EmptyNonce);
        }

        Ok(Self {
            client_id,
            redirect_uri,
            state,
            nonce,
            code_challenge,
        })
    }
}

fn is_canonical_s256_code_challenge(value: &str) -> bool {
    value.len() == S256_CODE_CHALLENGE_LENGTH
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn valid_raw_request() -> RawAuthorizationRequest {
        serde_json::from_value(json!({
            "response_type": "code",
            "client_id": "public-client-id",
            "redirect_uri": "https://client.example/callback?next=%2Fhome",
            "scope": "openid",
            "state": "state-with-exact-value",
            "nonce": "nonce-with-exact-value",
            "code_challenge": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "code_challenge_method": "S256"
        }))
        .expect("raw request should deserialize")
    }

    // NOTE: AI-generated test
    #[test]
    fn validates_v1_request_and_preserves_opaque_values() {
        let request = AuthorizationRequest::try_from(valid_raw_request())
            .expect("v1 authorization request should validate");

        assert_eq!(request.response_type(), "code");
        assert_eq!(request.client_id(), "public-client-id");
        assert_eq!(
            request.redirect_uri(),
            "https://client.example/callback?next=%2Fhome"
        );
        assert_eq!(request.scope(), "openid");
        assert_eq!(request.state(), "state-with-exact-value");
        assert_eq!(request.nonce(), Some("nonce-with-exact-value"));
        assert_eq!(
            request.code_challenge(),
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        );
        assert_eq!(request.code_challenge_method(), "S256");
    }

    // NOTE: AI-generated test
    #[test]
    fn accepts_an_omitted_nonce() {
        let mut raw = valid_raw_request();
        raw.nonce = None;

        let request = AuthorizationRequest::try_from(raw)
            .expect("nonce is optional in the authorization-code flow");

        assert_eq!(request.nonce(), None);
    }

    // NOTE: AI-generated test
    #[test]
    fn returns_deterministic_errors_for_invalid_request_local_parameters() {
        type Case = (
            &'static str,
            fn(&mut RawAuthorizationRequest),
            AuthorizationRequestError,
        );

        let cases: Vec<Case> = vec![
            (
                "missing response type",
                |raw| raw.response_type = None,
                AuthorizationRequestError::MissingResponseType,
            ),
            (
                "unsupported response type",
                |raw| raw.response_type = Some("token".to_owned()),
                AuthorizationRequestError::UnsupportedResponseType,
            ),
            (
                "missing client id",
                |raw| raw.client_id = None,
                AuthorizationRequestError::MissingClientId,
            ),
            (
                "empty client id",
                |raw| raw.client_id = Some(String::new()),
                AuthorizationRequestError::EmptyClientId,
            ),
            (
                "missing redirect uri",
                |raw| raw.redirect_uri = None,
                AuthorizationRequestError::MissingRedirectUri,
            ),
            (
                "relative redirect uri",
                |raw| raw.redirect_uri = Some("/callback".to_owned()),
                AuthorizationRequestError::InvalidRedirectUri,
            ),
            (
                "whitespace-padded redirect uri",
                |raw| raw.redirect_uri = Some(" https://client.example/callback ".to_owned()),
                AuthorizationRequestError::InvalidRedirectUri,
            ),
            (
                "redirect uri containing a control character",
                |raw| raw.redirect_uri = Some("https://client.example/call\nback".to_owned()),
                AuthorizationRequestError::InvalidRedirectUri,
            ),
            (
                "redirect uri fragment",
                |raw| raw.redirect_uri = Some("https://client.example/callback#done".to_owned()),
                AuthorizationRequestError::RedirectUriFragmentNotAllowed,
            ),
            (
                "missing scope",
                |raw| raw.scope = None,
                AuthorizationRequestError::MissingScope,
            ),
            (
                "unsupported scope",
                |raw| raw.scope = Some("openid profile".to_owned()),
                AuthorizationRequestError::UnsupportedScope,
            ),
            (
                "missing state",
                |raw| raw.state = None,
                AuthorizationRequestError::MissingState,
            ),
            (
                "empty state",
                |raw| raw.state = Some(String::new()),
                AuthorizationRequestError::EmptyState,
            ),
            (
                "missing code challenge method",
                |raw| raw.code_challenge_method = None,
                AuthorizationRequestError::MissingCodeChallengeMethod,
            ),
            (
                "plain code challenge method",
                |raw| raw.code_challenge_method = Some("plain".to_owned()),
                AuthorizationRequestError::UnsupportedCodeChallengeMethod,
            ),
            (
                "missing code challenge",
                |raw| raw.code_challenge = None,
                AuthorizationRequestError::MissingCodeChallenge,
            ),
            (
                "short code challenge",
                |raw| raw.code_challenge = Some("A".repeat(42)),
                AuthorizationRequestError::InvalidCodeChallenge,
            ),
            (
                "long code challenge",
                |raw| raw.code_challenge = Some("A".repeat(44)),
                AuthorizationRequestError::InvalidCodeChallenge,
            ),
            (
                "padded code challenge",
                |raw| raw.code_challenge = Some(format!("{}=", "A".repeat(42))),
                AuthorizationRequestError::InvalidCodeChallenge,
            ),
            (
                "non-base64url code challenge",
                |raw| raw.code_challenge = Some(format!("{}.", "A".repeat(42))),
                AuthorizationRequestError::InvalidCodeChallenge,
            ),
            (
                "empty nonce",
                |raw| raw.nonce = Some(String::new()),
                AuthorizationRequestError::EmptyNonce,
            ),
        ];

        for (name, mutate, expected) in cases {
            let mut raw = valid_raw_request();
            mutate(&mut raw);

            assert_eq!(
                AuthorizationRequest::try_from(raw),
                Err(expected),
                "case: {name}"
            );
        }
    }

    // NOTE: AI-generated test
    #[test]
    fn missing_fields_remain_available_to_typed_validation() {
        let raw: RawAuthorizationRequest =
            serde_json::from_value(json!({})).expect("missing fields should deserialize");

        assert_eq!(
            AuthorizationRequest::try_from(raw),
            Err(AuthorizationRequestError::MissingResponseType)
        );
    }
}
