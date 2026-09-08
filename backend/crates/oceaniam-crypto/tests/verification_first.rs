use oceaniam_crypto::{Algorithm, Validation, decode_rsa_der};

// NOTE: AI-generated test
#[test]
fn verification_is_safe_as_the_first_provider_dependent_operation() {
    let tokens: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/legacy-rust-crypto/tokens.json"))
            .expect("parse legacy token fixtures");
    let token = tokens["RS256"].as_str().expect("RS256 fixture token");
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&["https://legacy.oceaniam.test"]);
    validation.set_audience(&["legacy-client"]);
    let claims = decode_rsa_der::<serde_json::Value>(
        token,
        include_bytes!("fixtures/legacy-rust-crypto/public-key.pkcs1.der"),
        &validation,
    )
    .expect("verification wrapper should initialize AWS-LC before use");

    assert_eq!(claims.claims["sub"], "018f47a6-7b53-7cc0-8f5c-5c8bf2d6b8cf");
}
