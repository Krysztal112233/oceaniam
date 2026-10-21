//! OpenID Provider Configuration metadata document (OIDC Discovery 1.0 §3, RFC 8414 §2).
//!
//! Hand-rolled instead of `openidconnect::CoreProviderMetadata`: that type forces metadata
//! fields this provider deliberately does not advertise. The document mirrors exactly the
//! implemented capability — `authorization_code` grant only, public clients with mandatory
//! PKCE S256, `public` subject type, PS512 ID tokens, and the sub-only UserInfo claim set —
//! and nothing aspirational.

use serde::Serialize;
use utoipa::ToSchema;

/// OpenID Provider Configuration served from the tenant well-known paths.
///
/// Constructed from the canonical tenant issuer URL (`{public_base_url}/oidc/{tenant_sqid}`);
/// every endpoint URL is derived by appending the fixed endpoint suffix to the issuer, which
/// must not carry a trailing slash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct OpenIdProviderConfiguration {
    /// Canonical tenant issuer URL.
    pub issuer: String,
    /// `{issuer}/authorize`
    pub authorization_endpoint: String,
    /// `{issuer}/token`
    pub token_endpoint: String,
    /// `{issuer}/userinfo`
    pub userinfo_endpoint: String,
    /// `{issuer}/jwks.json`
    pub jwks_uri: String,
    /// Only the authorization code flow is implemented.
    pub response_types_supported: Vec<String>,
    /// No refresh token exists; `authorization_code` is the only grant.
    pub grant_types_supported: Vec<String>,
    /// `users.oidc_sub` is a stable per-user value, not per-client.
    pub subject_types_supported: Vec<String>,
    /// Tenant keyboxes sign ID tokens with PS512.
    pub id_token_signing_alg_values_supported: Vec<String>,
    /// Only the `openid` scope is accepted.
    pub scopes_supported: Vec<String>,
    /// Clients are public: the token request takes `client_id` only, no secret.
    pub token_endpoint_auth_methods_supported: Vec<String>,
    /// PKCE S256 is mandatory for every authorization request.
    pub code_challenge_methods_supported: Vec<String>,
    /// OIDC Discovery 1.0 §3 defaults this to `true` when omitted, but the authorization entry
    /// rejects every `request_uri` parameter with `RequestUriNotSupported`, so the document
    /// must advertise `false` explicitly.
    pub request_uri_parameter_supported: bool,
    /// The exact claim set an ID token can carry.
    pub claims_supported: Vec<String>,
}

impl OpenIdProviderConfiguration {
    /// Builds the configuration document for a tenant issuer.
    ///
    /// `issuer` is the canonical tenant issuer URL without a trailing slash, as produced by
    /// the authorization entry's `tenant_issuer`.
    pub fn new(issuer: impl Into<String>) -> Self {
        let issuer = issuer.into();
        debug_assert!(
            !issuer.ends_with('/'),
            "issuer must not carry a trailing slash"
        );
        Self {
            authorization_endpoint: format!("{issuer}/authorize"),
            token_endpoint: format!("{issuer}/token"),
            userinfo_endpoint: format!("{issuer}/userinfo"),
            jwks_uri: format!("{issuer}/jwks.json"),
            issuer,
            response_types_supported: vec!["code".to_owned()],
            grant_types_supported: vec!["authorization_code".to_owned()],
            subject_types_supported: vec!["public".to_owned()],
            id_token_signing_alg_values_supported: vec!["PS512".to_owned()],
            scopes_supported: vec!["openid".to_owned()],
            token_endpoint_auth_methods_supported: vec!["none".to_owned()],
            code_challenge_methods_supported: vec!["S256".to_owned()],
            request_uri_parameter_supported: false,
            claims_supported: ["iss", "sub", "aud", "exp", "iat", "auth_time", "nonce"]
                .iter()
                .map(|claim| (*claim).to_owned())
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: AI-generated test
    #[test]
    fn derives_all_endpoint_urls_from_the_issuer() {
        let configuration = OpenIdProviderConfiguration::new("https://iam.example.com/oidc/t1");

        assert_eq!(configuration.issuer, "https://iam.example.com/oidc/t1");
        assert_eq!(
            configuration.authorization_endpoint,
            "https://iam.example.com/oidc/t1/authorize"
        );
        assert_eq!(
            configuration.token_endpoint,
            "https://iam.example.com/oidc/t1/token"
        );
        assert_eq!(
            configuration.userinfo_endpoint,
            "https://iam.example.com/oidc/t1/userinfo"
        );
        assert_eq!(
            configuration.jwks_uri,
            "https://iam.example.com/oidc/t1/jwks.json"
        );
    }

    // NOTE: AI-generated test
    #[test]
    fn serializes_exactly_the_advertised_capability_and_nothing_more() {
        let configuration = OpenIdProviderConfiguration::new("https://iam.example.com/oidc/t1");
        let serialized = serde_json::to_value(&configuration).expect("configuration serializes");
        let object = serialized.as_object().expect("bare object");

        let expected_keys = [
            "issuer",
            "authorization_endpoint",
            "token_endpoint",
            "userinfo_endpoint",
            "jwks_uri",
            "response_types_supported",
            "grant_types_supported",
            "subject_types_supported",
            "id_token_signing_alg_values_supported",
            "scopes_supported",
            "token_endpoint_auth_methods_supported",
            "code_challenge_methods_supported",
            "request_uri_parameter_supported",
            "claims_supported",
        ];
        assert_eq!(
            object.len(),
            expected_keys.len(),
            "no fields beyond the advertised set: {serialized}"
        );
        for key in expected_keys {
            assert!(object.contains_key(key), "missing field `{key}`");
        }

        assert_eq!(
            serialized["response_types_supported"],
            serde_json::json!(["code"])
        );
        assert_eq!(
            serialized["grant_types_supported"],
            serde_json::json!(["authorization_code"])
        );
        assert_eq!(
            serialized["subject_types_supported"],
            serde_json::json!(["public"])
        );
        assert_eq!(
            serialized["id_token_signing_alg_values_supported"],
            serde_json::json!(["PS512"])
        );
        assert_eq!(
            serialized["scopes_supported"],
            serde_json::json!(["openid"])
        );
        assert_eq!(
            serialized["token_endpoint_auth_methods_supported"],
            serde_json::json!(["none"])
        );
        assert_eq!(
            serialized["code_challenge_methods_supported"],
            serde_json::json!(["S256"])
        );
        assert_eq!(
            serialized["request_uri_parameter_supported"],
            serde_json::json!(false)
        );
        assert_eq!(
            serialized["claims_supported"],
            serde_json::json!(["iss", "sub", "aud", "exp", "iat", "auth_time", "nonce"])
        );
    }
}
