//! Transport types and request-local validation for the OIDC Authorization Endpoint.
//!
//! This module deliberately does not validate client registration, tenant ownership, or exact
//! redirect URI registration, and it does not decide whether an error may be returned by redirect.
//! HTTP callers must establish those trust properties before converting a parsed request into the
//! validated request-local profile.

use serde::Deserialize;
use url::Url;

use super::pkce::decode_s256_code_challenge;
use crate::error::{AuthorizationProtocolError, AuthorizationRequestError};

/// Maximum encoded query/form payload accepted by the Authorization Endpoint.
pub const AUTHORIZATION_FORM_MAX_BYTES: usize = 8 * 1024;

const RESPONSE_TYPE_CODE: &str = "code";
const SCOPE_OPENID: &str = "openid";
const CODE_CHALLENGE_METHOD_S256: &str = "S256";

/// Authorization Endpoint parameters as deserialized from the request transport.
///
/// Fields are optional so request-local validation can report deterministic missing-parameter
/// errors; empty values deserialize as omission. Deserialization is lossy (WHATWG form URL
/// decoding): malformed percent sequences or UTF-8 are carried verbatim, with downstream
/// exact-match validation as the security backstop.
///
/// Controls this profile cannot honor are handled per the base specification:
/// `request`/`request_uri`/`registration` are retained for the deferred protocol error
/// surfaced by [`ParsedAuthorizationRequest`]; a non-default `response_mode` is flagged early
/// for a bare HTTP 400 (OIDC Core 1.0 §5: the encoding mode needed to carry an Error Response
/// is unknown); unsupported optional parameters without a mandated behavior (`max_age`,
/// `claims`, `id_token_hint`, `acr_values`) are ignored like unknown extensions (OIDC Core
/// 1.0 §5.5 ignores an unsupported `claims` parameter rather than erroring).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct RawAuthorizationRequest {
    response_type: Option<String>,
    client_id: Option<String>,
    redirect_uri: Option<String>,
    scope: Option<String>,
    state: Option<String>,
    nonce: Option<String>,
    code_challenge: Option<String>,
    code_challenge_method: Option<String>,
    response_mode: Option<String>,
    request: Option<String>,
    request_uri: Option<String>,
    registration: Option<String>,
    prompt: Option<String>,
}

/// Deserialized authorization parameters awaiting callback trust and protocol checks.
///
/// This type deliberately omits `Debug`: it can contain opaque state and nonce values.
/// Recognized unsupported controls are retained as a deferred protocol error.
pub struct ParsedAuthorizationRequest {
    raw: RawAuthorizationRequest,
    deferred_error: Option<AuthorizationProtocolError>,
}

impl From<RawAuthorizationRequest> for ParsedAuthorizationRequest {
    /// Captures recognized controls this profile cannot honor honestly as a deferred protocol
    /// error. The error is surfaced only after client and callback trust is established, so the
    /// trust ordering of protocol-error redirects never changes.
    fn from(raw: RawAuthorizationRequest) -> Self {
        let deferred_error = if raw.request.is_some() {
            Some(AuthorizationProtocolError::RequestNotSupported)
        } else if raw.request_uri.is_some() {
            Some(AuthorizationProtocolError::RequestUriNotSupported)
        } else if raw.registration.is_some() {
            Some(AuthorizationProtocolError::RegistrationNotSupported)
        } else if raw.prompt.is_some() {
            // OIDC Core 1.0 §3.1.2.1 permits an error for a `prompt` this OP cannot honor
            // (MAY return an error); silently ignoring it would misrepresent `prompt=none`.
            Some(AuthorizationProtocolError::InvalidRequest)
        } else {
            None
        };

        Self {
            raw,
            deferred_error,
        }
    }
}

impl ParsedAuthorizationRequest {
    pub fn client_id(&self) -> Option<&str> {
        self.raw.client_id.as_deref()
    }

    pub fn redirect_uri(&self) -> Option<&str> {
        self.raw.redirect_uri.as_deref()
    }

    pub fn state(&self) -> Option<&str> {
        self.raw.state.as_deref()
    }

    /// Whether the request asks for a Response Mode other than the supported default `query`.
    /// OIDC Core 1.0 §5 requires this to be a bare HTTP 400 without Error Response parameters
    /// — the unsupported mode is exactly what would encode them — so callers must reject it
    /// before any callback-trust work instead of deferring a protocol error.
    pub fn has_unsupported_response_mode(&self) -> bool {
        self.raw
            .response_mode
            .as_deref()
            .is_some_and(|mode| mode != "query")
    }

    /// Applies request-local protocol validation. Callers must first establish tenant ownership
    /// and exact callback trust under the live, locked registration policy.
    pub fn into_authorization_request(
        self,
    ) -> Result<AuthorizationRequest, AuthorizationProtocolError> {
        if let Some(error) = self.deferred_error {
            return Err(error);
        }

        AuthorizationRequest::try_from(self.raw).map_err(Into::into)
    }
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
            ..
        } = raw;

        let response_type = response_type.ok_or(AuthorizationRequestError::MissingResponseType)?;
        if response_type != RESPONSE_TYPE_CODE {
            return Err(AuthorizationRequestError::UnsupportedResponseType);
        }

        let client_id = client_id.ok_or(AuthorizationRequestError::MissingClientId)?;
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

        let code_challenge_method =
            code_challenge_method.ok_or(AuthorizationRequestError::MissingCodeChallengeMethod)?;
        if code_challenge_method != CODE_CHALLENGE_METHOD_S256 {
            return Err(AuthorizationRequestError::UnsupportedCodeChallengeMethod);
        }

        let code_challenge =
            code_challenge.ok_or(AuthorizationRequestError::MissingCodeChallenge)?;
        if decode_s256_code_challenge(&code_challenge).is_none() {
            return Err(AuthorizationRequestError::InvalidCodeChallenge);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_raw_request() -> RawAuthorizationRequest {
        RawAuthorizationRequest {
            response_type: Some("code".to_owned()),
            client_id: Some("public-client-id".to_owned()),
            redirect_uri: Some("https://client.example/callback?next=%2Fhome".to_owned()),
            scope: Some("openid".to_owned()),
            state: Some("state-with-exact-value".to_owned()),
            nonce: Some("nonce-with-exact-value".to_owned()),
            code_challenge: Some("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_owned()),
            code_challenge_method: Some("S256".to_owned()),
            ..Default::default()
        }
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
                "non-canonical base64url trailing bits",
                |raw| raw.code_challenge = Some(format!("{}B", "A".repeat(42))),
                AuthorizationRequestError::InvalidCodeChallenge,
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
        let parsed = ParsedAuthorizationRequest::from(RawAuthorizationRequest::default());

        assert_eq!(
            parsed.into_authorization_request(),
            Err(AuthorizationProtocolError::InvalidRequest)
        );
    }

    // NOTE: AI-generated test
    #[test]
    fn recognized_unsupported_parameters_are_deferred_protocol_errors() {
        type Case = (
            &'static str,
            fn(&mut RawAuthorizationRequest),
            AuthorizationProtocolError,
        );

        let cases: Vec<Case> = vec![
            (
                "request object",
                |raw| raw.request = Some("jwt".to_owned()),
                AuthorizationProtocolError::RequestNotSupported,
            ),
            (
                "request uri",
                |raw| raw.request_uri = Some("https://client.example/request.jwt".to_owned()),
                AuthorizationProtocolError::RequestUriNotSupported,
            ),
            (
                "registration",
                |raw| raw.registration = Some("eyJmb28iOiJiYXIifQ".to_owned()),
                AuthorizationProtocolError::RegistrationNotSupported,
            ),
            (
                "prompt",
                |raw| raw.prompt = Some("login".to_owned()),
                AuthorizationProtocolError::InvalidRequest,
            ),
        ];

        for (name, mutate, expected) in cases {
            let mut raw = valid_raw_request();
            mutate(&mut raw);

            assert_eq!(
                ParsedAuthorizationRequest::from(raw).into_authorization_request(),
                Err(expected),
                "case: {name}"
            );
        }

        let query_mode = ParsedAuthorizationRequest::from(RawAuthorizationRequest {
            response_mode: Some("query".to_owned()),
            ..valid_raw_request()
        });
        assert!(query_mode.into_authorization_request().is_ok());
    }

    // NOTE: AI-generated test
    #[test]
    fn unsupported_response_modes_are_flagged_for_a_bare_local_400() {
        for mode in ["fragment", "form_post"] {
            let parsed = ParsedAuthorizationRequest::from(RawAuthorizationRequest {
                response_mode: Some(mode.to_owned()),
                ..valid_raw_request()
            });
            assert!(parsed.has_unsupported_response_mode(), "mode: {mode}");
        }

        // An empty value is normalized to omission by the form deserializer before a
        // `RawAuthorizationRequest` exists, so only the present-nonempty case is flagged.
        for mode in [None, Some("query")] {
            let parsed = ParsedAuthorizationRequest::from(RawAuthorizationRequest {
                response_mode: mode.map(ToOwned::to_owned),
                ..valid_raw_request()
            });
            assert!(!parsed.has_unsupported_response_mode(), "mode: {mode:?}");
        }
    }

    // NOTE: AI-generated test
    #[test]
    fn authorization_protocol_errors_use_the_allowlisted_wire_codes() {
        for (error, expected) in [
            (
                AuthorizationProtocolError::InvalidRequest,
                "invalid_request",
            ),
            (
                AuthorizationProtocolError::UnsupportedResponseType,
                "unsupported_response_type",
            ),
            (AuthorizationProtocolError::InvalidScope, "invalid_scope"),
            (
                AuthorizationProtocolError::RequestNotSupported,
                "request_not_supported",
            ),
            (
                AuthorizationProtocolError::RequestUriNotSupported,
                "request_uri_not_supported",
            ),
            (
                AuthorizationProtocolError::RegistrationNotSupported,
                "registration_not_supported",
            ),
            (AuthorizationProtocolError::ServerError, "server_error"),
        ] {
            assert_eq!(error.as_str(), expected);
        }
    }
}
