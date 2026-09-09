use url::Url;

use crate::error::RedirectUriError;

/// Maximum byte length accepted for a registered redirect URI.
pub const MAX_REDIRECT_URI_LENGTH: usize = 2048;

/// Registration-time policy for OceanIAM's web-client redirect URIs.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RedirectUriPolicy {
    allow_insecure_loopback_redirect_uris: bool,
}

impl RedirectUriPolicy {
    pub const fn web() -> Self {
        Self {
            allow_insecure_loopback_redirect_uris: false,
        }
    }

    pub const fn web_with_insecure_loopback() -> Self {
        Self {
            allow_insecure_loopback_redirect_uris: true,
        }
    }
}

/// Validates a redirect URI for registration without normalizing the supplied string.
///
/// Callers must persist and compare the original string so Authorization Endpoint matching can use
/// exact string comparison. The development exception permits `http` only for `localhost`,
/// `127.0.0.1`, or `[::1]`; it does not enable the native-client variable-port
/// exception.
pub fn validate_redirect_uri(
    value: &str,
    policy: RedirectUriPolicy,
) -> Result<(), RedirectUriError> {
    if value.is_empty()
        || !value.is_ascii()
        || value
            .bytes()
            .any(|byte| byte.is_ascii_control() || matches!(byte, b' ' | b'\\'))
        || raw_authority(value).is_none()
    {
        return Err(RedirectUriError::Invalid);
    }

    if value.len() > MAX_REDIRECT_URI_LENGTH {
        return Err(RedirectUriError::TooLong);
    }

    let parsed = Url::parse(value).map_err(|_| RedirectUriError::Invalid)?;
    if parsed.host().is_none() {
        return Err(RedirectUriError::Invalid);
    }

    if raw_authority(value).is_some_and(|authority| authority.contains('@'))
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return Err(RedirectUriError::CredentialsNotAllowed);
    }

    if parsed.query().is_some() {
        return Err(RedirectUriError::QueryNotAllowed);
    }

    if parsed.fragment().is_some() {
        return Err(RedirectUriError::FragmentNotAllowed);
    }

    match parsed.scheme() {
        "https" => Ok(()),
        "http" if !policy.allow_insecure_loopback_redirect_uris => {
            Err(RedirectUriError::HttpsRequired)
        }
        "http" if has_allowed_loopback_authority(value) => Ok(()),
        "http" => Err(RedirectUriError::LoopbackHostRequired),
        _ => Err(RedirectUriError::HttpsRequired),
    }
}

fn raw_authority(value: &str) -> Option<&str> {
    let scheme_end = value.find(':')?;
    let remainder = value.get(scheme_end + 1..)?.strip_prefix("//")?;

    remainder
        .split(['/', '?', '#'])
        .next()
        .filter(|authority| !authority.is_empty())
}

fn has_allowed_loopback_authority(value: &str) -> bool {
    let Some(authority) = raw_authority(value) else {
        return false;
    };

    if let Some(closing_bracket) = authority.find(']') {
        let (host, port) = authority.split_at(closing_bracket + 1);
        return host.eq_ignore_ascii_case("[::1]") && valid_optional_port(port);
    }

    let (host, port) = authority
        .rsplit_once(':')
        .map_or((authority, ""), |(host, _)| {
            (host, &authority[host.len()..])
        });

    (host.eq_ignore_ascii_case("localhost") || host == "127.0.0.1") && valid_optional_port(port)
}

fn valid_optional_port(value: &str) -> bool {
    value.is_empty()
        || value
            .strip_prefix(':')
            .is_some_and(|port| !port.is_empty() && port.bytes().all(|byte| byte.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: AI-generated test
    #[test]
    fn accepts_https_web_redirect_uris_without_normalizing_them() {
        for value in [
            "https://client.example/callback",
            "https://client.example:8443/oidc/callback",
            "https://client.example/callback/%2Fencoded",
        ] {
            assert_eq!(
                validate_redirect_uri(value, RedirectUriPolicy::web()),
                Ok(()),
                "unexpectedly rejected {value}"
            );
        }
    }

    // NOTE: AI-generated test
    #[test]
    fn rejects_ambiguous_or_unsupported_web_redirect_uris() {
        let cases = [
            ("", RedirectUriError::Invalid),
            ("/callback", RedirectUriError::Invalid),
            ("https:///callback", RedirectUriError::Invalid),
            ("https:client.example/://x", RedirectUriError::Invalid),
            (
                "https://user:password@client.example/callback",
                RedirectUriError::CredentialsNotAllowed,
            ),
            (
                "https://@client.example/callback",
                RedirectUriError::CredentialsNotAllowed,
            ),
            (
                "https://client.example/callback?next=%2Fhome",
                RedirectUriError::QueryNotAllowed,
            ),
            (
                "https://client.example/callback#done",
                RedirectUriError::FragmentNotAllowed,
            ),
            (
                "http://localhost:3000/callback",
                RedirectUriError::HttpsRequired,
            ),
            ("com.example.client:/callback", RedirectUriError::Invalid),
            (
                "https://client.example/call back",
                RedirectUriError::Invalid,
            ),
            (
                "https://evil.example\\@client.example/callback",
                RedirectUriError::Invalid,
            ),
            ("https://client.example/回调", RedirectUriError::Invalid),
        ];

        for (value, expected) in cases {
            assert_eq!(
                validate_redirect_uri(value, RedirectUriPolicy::web()),
                Err(expected),
                "unexpected result for {value}"
            );
        }
    }

    // NOTE: AI-generated test
    #[test]
    fn development_policy_allows_only_fixed_loopback_http_uris() {
        let policy = RedirectUriPolicy::web_with_insecure_loopback();

        for value in [
            "http://localhost:3000/callback",
            "http://127.0.0.1:8080/callback",
            "http://[::1]:4173/callback",
        ] {
            assert_eq!(
                validate_redirect_uri(value, policy),
                Ok(()),
                "unexpectedly rejected {value}"
            );
        }

        for value in [
            "http://example.com/callback",
            "http://sub.localhost/callback",
            "http://localhost.example/callback",
            "http://localhost./callback",
            "http://127.0.0.2/callback",
            "http://127.1/callback",
            "http://2130706433/callback",
            "http://0x7f000001/callback",
            "http://0177.0.0.1/callback",
            "http://%31%32%37.0.0.1/callback",
            "http://0.0.0.0/callback",
            "http://[::2]/callback",
            "http://[0:0:0:0:0:0:0:1]/callback",
            "http://local%68ost/callback",
        ] {
            assert_eq!(
                validate_redirect_uri(value, policy),
                Err(RedirectUriError::LoopbackHostRequired),
                "unexpected result for {value}"
            );
        }
    }

    // NOTE: AI-generated test
    #[test]
    fn enforces_the_redirect_uri_storage_boundary() {
        let prefix = "https://client.example/";
        let exact = format!(
            "{prefix}{}",
            "a".repeat(MAX_REDIRECT_URI_LENGTH - prefix.len())
        );
        let too_long = format!("{exact}a");

        assert_eq!(exact.len(), MAX_REDIRECT_URI_LENGTH);
        assert_eq!(
            validate_redirect_uri(&exact, RedirectUriPolicy::web()),
            Ok(())
        );
        assert_eq!(too_long.len(), MAX_REDIRECT_URI_LENGTH + 1);
        assert_eq!(
            validate_redirect_uri(&too_long, RedirectUriPolicy::web()),
            Err(RedirectUriError::TooLong)
        );
    }
}
