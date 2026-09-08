use oceaniam_crypto::{Algorithm, Header, encode_rsa_der};

// NOTE: AI-generated test
#[test]
fn signing_is_safe_as_the_first_provider_dependent_operation() {
    let token = encode_rsa_der(
        &Header::new(Algorithm::RS256),
        &serde_json::json!({"sub": "signing-first", "exp": 4_102_444_800_i64}),
        include_bytes!("fixtures/legacy-rust-crypto/private-key.pkcs1.der"),
    )
    .expect("signing wrapper should initialize AWS-LC before use");

    assert_eq!(token.split('.').count(), 3);
}
