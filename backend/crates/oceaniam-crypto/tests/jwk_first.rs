use oceaniam_crypto::{Algorithm, rsa_public_jwk_from_private_der};

// NOTE: AI-generated test
#[test]
fn jwk_derivation_is_safe_as_the_first_provider_dependent_operation() {
    let jwk = rsa_public_jwk_from_private_der(
        include_bytes!("fixtures/legacy-rust-crypto/private-key.pkcs1.der"),
        Algorithm::PS512,
    )
    .expect("JWK wrapper should initialize AWS-LC before use");
    let value = serde_json::to_value(jwk).expect("serialize derived JWK");

    assert_eq!(value["kty"], "RSA");
    assert_eq!(value["alg"], "PS512");
}
