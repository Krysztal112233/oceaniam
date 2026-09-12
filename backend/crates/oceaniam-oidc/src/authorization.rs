//! Strict transport parsing and request-local validation for the OIDC Authorization Endpoint.
//!
//! This module deliberately does not validate client registration, tenant ownership, or exact
//! redirect URI registration, and it does not decide whether an error may be returned by redirect.
//! HTTP callers must establish those trust properties before converting a parsed request into the
//! validated request-local profile.

use std::collections::HashSet;

use serde::Deserialize;
use url::Url;

use super::pkce::decode_s256_code_challenge;
use crate::error::{AuthorizationFormError, AuthorizationProtocolError, AuthorizationRequestError};

/// Maximum encoded query/form payload accepted by the Authorization Endpoint.
pub const AUTHORIZATION_FORM_MAX_BYTES: usize = 8 * 1024;

const RESPONSE_TYPE_CODE: &str = "code";
const SCOPE_OPENID: &str = "openid";
const CODE_CHALLENGE_METHOD_S256: &str = "S256";

/// Deserializable Authorization Endpoint parameters before protocol validation.
///
/// Fields are optional so request-local validation can report deterministic missing-parameter
/// errors. Public HTTP input reaches this type only through [`parse_authorization_form`], which
/// supplies the strict duplicate, encoding, empty-value, and extension handling boundary.
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

/// Strictly once-decoded authorization parameters awaiting callback trust and protocol checks.
///
/// This type deliberately omits `Debug`: it can contain opaque state and nonce values. Empty
/// values have already been normalized to omission, unknown extensions ignored, and recognized
/// unsupported controls retained as a deferred protocol error.
pub struct ParsedAuthorizationRequest {
    raw: RawAuthorizationRequest,
    deferred_error: Option<AuthorizationProtocolError>,
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

/// Parses an encoded authorization query or form body without lossy or repeated decoding.
///
/// Duplicate names are detected after decoding and before empty/unknown fields are discarded.
/// Every name and value, including ignored extensions, receives strict percent, UTF-8, and NUL
/// validation.
pub fn parse_authorization_form(
    encoded: &[u8],
) -> Result<ParsedAuthorizationRequest, AuthorizationFormError> {
    if encoded.len() > AUTHORIZATION_FORM_MAX_BYTES {
        return Err(AuthorizationFormError::FormTooLong);
    }

    let mut names = HashSet::new();
    let mut response_type = None;
    let mut client_id = None;
    let mut redirect_uri = None;
    let mut scope = None;
    let mut state = None;
    let mut nonce = None;
    let mut code_challenge = None;
    let mut code_challenge_method = None;
    let mut deferred_error = None;

    for pair in encoded.split(|byte| *byte == b'&') {
        if pair.is_empty() {
            continue;
        }

        let separator = pair.iter().position(|byte| *byte == b'=');
        let (encoded_name, encoded_value) = match separator {
            Some(index) => (&pair[..index], &pair[index + 1..]),
            None => (pair, &[][..]),
        };
        let name = decode_form_component(encoded_name)?;
        let value = decode_form_component(encoded_value)?;

        if !names.insert(name.clone()) {
            return Err(AuthorizationFormError::DuplicateParameter);
        }

        // RFC 6749 treats empty values as omitted. Duplicate detection intentionally happens
        // first so empty or unknown fields cannot conceal a duplicate decoded name.
        if value.is_empty() {
            continue;
        }

        match name.as_str() {
            "response_type" => response_type = Some(value),
            "client_id" => client_id = Some(value),
            "redirect_uri" => redirect_uri = Some(value),
            "scope" => scope = Some(value),
            "state" => state = Some(value),
            "nonce" => nonce = Some(value),
            "code_challenge" => code_challenge = Some(value),
            "code_challenge_method" => code_challenge_method = Some(value),
            "response_mode" if value == "query" => {}
            "request" => {
                deferred_error.get_or_insert(AuthorizationProtocolError::RequestNotSupported);
            }
            "request_uri" => {
                deferred_error.get_or_insert(AuthorizationProtocolError::RequestUriNotSupported);
            }
            "registration" => {
                deferred_error.get_or_insert(AuthorizationProtocolError::RegistrationNotSupported);
            }
            "response_mode" | "prompt" | "max_age" | "claims" | "id_token_hint" | "acr_values" => {
                deferred_error.get_or_insert(AuthorizationProtocolError::InvalidRequest);
            }
            // Display and login hints, plus genuinely unknown extensions, have no persisted
            // semantics in this preview and are intentionally ignored after strict decoding.
            _ => continue,
        };
    }

    Ok(ParsedAuthorizationRequest {
        raw: RawAuthorizationRequest {
            response_type,
            client_id,
            redirect_uri,
            scope,
            state,
            nonce,
            code_challenge,
            code_challenge_method,
        },
        deferred_error,
    })
}

fn decode_form_component(encoded: &[u8]) -> Result<String, AuthorizationFormError> {
    let mut decoded = Vec::with_capacity(encoded.len());
    let mut index = 0;

    while index < encoded.len() {
        match encoded[index] {
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            b'%' => {
                let high = encoded
                    .get(index + 1)
                    .and_then(|byte| decode_hex(*byte))
                    .ok_or(AuthorizationFormError::InvalidPercentEncoding)?;
                let low = encoded
                    .get(index + 2)
                    .and_then(|byte| decode_hex(*byte))
                    .ok_or(AuthorizationFormError::InvalidPercentEncoding)?;
                decoded.push((high << 4) | low);
                index += 3;
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }

    if decoded.contains(&0) {
        return Err(AuthorizationFormError::NulNotAllowed);
    }

    String::from_utf8(decoded).map_err(|_| AuthorizationFormError::InvalidUtf8)
}

const fn decode_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
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
        if decode_s256_code_challenge(&code_challenge).is_none() {
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
                "non-canonical base64url trailing bits",
                |raw| raw.code_challenge = Some(format!("{}B", "A".repeat(42))),
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

    fn valid_encoded_request() -> &'static [u8] {
        b"response_type=code&client_id=public-client-id&redirect_uri=https%3A%2F%2Fclient.example%2Fcallback&scope=openid&state=opaque%252Fstate%2Bvalue&nonce=nonce&code_challenge=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA&code_challenge_method=S256"
    }

    // NOTE: AI-generated test
    #[test]
    fn strict_form_parser_is_transport_agnostic_and_decodes_opaque_values_once() {
        let get =
            parse_authorization_form(valid_encoded_request()).expect("GET query should parse");
        let post =
            parse_authorization_form(valid_encoded_request()).expect("POST form should parse");

        assert_eq!(get.client_id(), post.client_id());
        assert_eq!(get.redirect_uri(), post.redirect_uri());
        assert_eq!(get.state(), Some("opaque%2Fstate+value"));
        assert_eq!(post.state(), Some("opaque%2Fstate+value"));

        let request = get
            .into_authorization_request()
            .expect("request-local profile should validate after trust");
        assert_eq!(request.state(), "opaque%2Fstate+value");
    }

    // NOTE: AI-generated test
    #[test]
    fn strict_form_parser_rejects_decoded_duplicates_before_discarding_fields() {
        for encoded in [
            b"client_id=one&%63lient_id=two".as_slice(),
            b"unknown=&unknown=value".as_slice(),
            b"ignored=one&%69gnored=two".as_slice(),
        ] {
            assert!(matches!(
                parse_authorization_form(encoded),
                Err(AuthorizationFormError::DuplicateParameter)
            ));
        }
    }

    // NOTE: AI-generated test
    #[test]
    fn strict_form_parser_validates_every_name_and_value() {
        for (encoded, expected) in [
            (
                b"unknown=%".as_slice(),
                AuthorizationFormError::InvalidPercentEncoding,
            ),
            (
                b"unknown=%GG".as_slice(),
                AuthorizationFormError::InvalidPercentEncoding,
            ),
            (
                b"unknown=%ff".as_slice(),
                AuthorizationFormError::InvalidUtf8,
            ),
            (
                b"unk%00nown=value".as_slice(),
                AuthorizationFormError::NulNotAllowed,
            ),
            (
                b"unknown=value%00".as_slice(),
                AuthorizationFormError::NulNotAllowed,
            ),
        ] {
            assert_eq!(parse_authorization_form(encoded).err(), Some(expected));
        }
    }

    // NOTE: AI-generated test
    #[test]
    fn empty_and_unknown_values_do_not_change_the_request_profile() {
        let mut encoded = valid_encoded_request().to_vec();
        encoded.extend_from_slice(
            b"&display=popup&login_hint=user%40example.com&ui_locales=en&unknown=value&another=",
        );

        let request = parse_authorization_form(&encoded)
            .expect("ignorable extensions should parse")
            .into_authorization_request()
            .expect("ignorable extensions should not alter validation");

        assert_eq!(request.client_id(), "public-client-id");
        assert_eq!(request.nonce(), Some("nonce"));

        let without_nonce = valid_encoded_request()
            .strip_suffix(b"&nonce=nonce&code_challenge=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA&code_challenge_method=S256")
            .expect("fixture suffix");
        let mut empty_nonce = without_nonce.to_vec();
        empty_nonce.extend_from_slice(b"&nonce=&code_challenge=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA&code_challenge_method=S256");
        assert_eq!(
            parse_authorization_form(&empty_nonce)
                .expect("empty nonce should parse as omitted")
                .into_authorization_request()
                .expect("empty HTTP nonce should be omitted")
                .nonce(),
            None
        );
    }

    // NOTE: AI-generated test
    #[test]
    fn recognized_unsupported_parameters_are_deferred_protocol_errors() {
        let cases = [
            (
                "request=jwt",
                AuthorizationProtocolError::RequestNotSupported,
            ),
            (
                "request_uri=https%3A%2F%2Fclient.example%2Frequest.jwt",
                AuthorizationProtocolError::RequestUriNotSupported,
            ),
            (
                "registration=eyJmb28iOiJiYXIifQ",
                AuthorizationProtocolError::RegistrationNotSupported,
            ),
            ("prompt=login", AuthorizationProtocolError::InvalidRequest),
            ("max_age=0", AuthorizationProtocolError::InvalidRequest),
            ("claims=%7B%7D", AuthorizationProtocolError::InvalidRequest),
            (
                "id_token_hint=token",
                AuthorizationProtocolError::InvalidRequest,
            ),
            (
                "acr_values=loa1",
                AuthorizationProtocolError::InvalidRequest,
            ),
            (
                "response_mode=fragment",
                AuthorizationProtocolError::InvalidRequest,
            ),
        ];

        for (parameter, expected) in cases {
            let mut encoded = valid_encoded_request().to_vec();
            encoded.extend_from_slice(b"&");
            encoded.extend_from_slice(parameter.as_bytes());
            let parsed = parse_authorization_form(&encoded).expect("transport should parse");
            assert_eq!(parsed.into_authorization_request(), Err(expected));
        }

        let mut query_mode = valid_encoded_request().to_vec();
        query_mode.extend_from_slice(b"&response_mode=query");
        assert!(
            parse_authorization_form(&query_mode)
                .expect("query response mode should parse")
                .into_authorization_request()
                .is_ok()
        );
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

    // NOTE: AI-generated test
    #[test]
    fn strict_form_parser_enforces_the_encoded_byte_boundary() {
        let exact = vec![b'x'; AUTHORIZATION_FORM_MAX_BYTES];
        let over = vec![b'x'; AUTHORIZATION_FORM_MAX_BYTES + 1];

        assert!(parse_authorization_form(&exact).is_ok());
        assert!(matches!(
            parse_authorization_form(&over),
            Err(AuthorizationFormError::FormTooLong)
        ));
    }
}
