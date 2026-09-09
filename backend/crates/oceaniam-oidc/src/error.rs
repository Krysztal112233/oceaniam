//! Protocol-specific validation and conversion errors.

use snafu::{Location, Snafu};

use crate::redirect_uri::MAX_REDIRECT_URI_LENGTH;

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

/// Deterministic failures while converting an internal JWK Set into the OIDC core type.
///
/// The conversion is deliberately strict: `openidconnect`'s `JsonWebKeySet` deserialization
/// silently skips keys it cannot parse, so the set is never deserialized wholesale. Each key is
/// validated and converted individually, and any invalid key fails the whole conversion.
#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum OidcJwksError {
    #[snafu(display("JWK #{index} has unsupported key type `{kty}` at {location}"))]
    UnsupportedKeyType {
        index: usize,
        kty: String,
        location: Location,
    },

    #[snafu(display("JWK #{index} is missing the `{field}` parameter at {location}"))]
    MissingField {
        index: usize,
        field: &'static str,
        location: Location,
    },

    #[snafu(display("JWK #{index} has an invalid base64url `{field}` value at {location}"))]
    InvalidBase64Url {
        index: usize,
        field: &'static str,
        location: Location,
    },

    #[snafu(display("JWK #{index} is not a valid core JWK at {location}"))]
    InvalidJwk {
        index: usize,
        source: serde_json::Error,
        location: Location,
    },
}

/// Deterministic PKCE S256 verification failures, without HTTP semantics or sensitive values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Snafu)]
#[non_exhaustive]
pub enum PkceS256Error {
    /// The verifier does not satisfy the RFC 7636 verifier grammar.
    #[snafu(display("code_verifier is not a valid PKCE verifier"))]
    InvalidCodeVerifier,

    /// The verifier does not match, or the stored challenge is not a canonical S256 challenge.
    #[snafu(display("PKCE verification failed"))]
    ChallengeMismatch,
}

/// Registration-time redirect URI validation failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Snafu)]
#[non_exhaustive]
pub enum RedirectUriError {
    #[snafu(display("redirect URI must be a non-empty absolute ASCII URI"))]
    Invalid,

    #[snafu(display("redirect URI must not exceed {MAX_REDIRECT_URI_LENGTH} ASCII characters"))]
    TooLong,

    #[snafu(display("redirect URI must not include credentials"))]
    CredentialsNotAllowed,

    #[snafu(display("redirect URI must not include a query"))]
    QueryNotAllowed,

    #[snafu(display("redirect URI must not include a fragment"))]
    FragmentNotAllowed,

    #[snafu(display("web-client redirect URI must use the `https` scheme"))]
    HttpsRequired,

    #[snafu(display("insecure development redirect URI must use a loopback host"))]
    LoopbackHostRequired,
}
