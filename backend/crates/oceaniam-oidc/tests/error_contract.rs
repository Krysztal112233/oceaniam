use std::error::Error as _;

use oceaniam_oidc::{AuthorizationRequestError, OidcJwksError, PkceS256Error, RedirectUriError};

fn assert_std_error<T: std::error::Error>() {}

// NOTE: AI-generated test
#[test]
fn error_types_remain_root_exported_with_stable_messages_and_sources() {
    assert_std_error::<AuthorizationRequestError>();
    assert_std_error::<OidcJwksError>();
    assert_std_error::<PkceS256Error>();
    assert_std_error::<RedirectUriError>();

    assert_eq!(
        AuthorizationRequestError::MissingResponseType.to_string(),
        "response_type is required"
    );
    assert_eq!(
        PkceS256Error::ChallengeMismatch.to_string(),
        "PKCE verification failed"
    );
    assert_eq!(
        RedirectUriError::TooLong.to_string(),
        "redirect URI must not exceed 2048 ASCII characters"
    );

    let source = serde_json::from_str::<serde_json::Value>("{")
        .expect_err("malformed JSON should produce a source error");
    let source_message = source.to_string();
    let error = OidcJwksError::InvalidJwk {
        index: 7,
        source,
        location: snafu::location!(),
    };

    assert_eq!(
        error.source().map(ToString::to_string),
        Some(source_message)
    );
    assert!(
        error
            .to_string()
            .contains("JWK #7 is not a valid core JWK at")
    );
    assert!(error.to_string().contains("tests/error_contract.rs"));
}
