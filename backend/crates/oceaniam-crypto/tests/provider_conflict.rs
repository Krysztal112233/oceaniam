use jsonwebtoken::crypto::{CryptoProvider, JwkUtils};
use oceaniam_crypto::{
    Algorithm, Header, ProviderJwk, RsaPrivateKey, Validation, decode_rsa_der,
    decoding_key_from_jwk, encode_rsa, encode_rsa_der, initialize_jwt_provider, rsa_public_jwk,
    rsa_public_jwk_from_private_der,
};

static SENTINEL_PROVIDER: CryptoProvider = CryptoProvider {
    signer_factory: |_, _| {
        Err(jsonwebtoken::errors::new_error(
            jsonwebtoken::errors::ErrorKind::InvalidAlgorithm,
        ))
    },
    verifier_factory: |_, _| {
        Err(jsonwebtoken::errors::new_error(
            jsonwebtoken::errors::ErrorKind::InvalidAlgorithm,
        ))
    },
    jwk_utils: JwkUtils::new_unimplemented(),
};

// NOTE: AI-generated test
#[test]
fn a_conflicting_provider_causes_persistent_fail_closed_errors() {
    let private_key = RsaPrivateKey::from_pkcs8_pem(include_bytes!(
        "fixtures/legacy-rust-crypto/private-key.pkcs8.pem"
    ))
    .expect("pure AWS-LC key loading should not require the JWT provider");

    SENTINEL_PROVIDER
        .install_default()
        .expect("sentinel should be the first provider in this test process");

    let first = initialize_jwt_provider().expect_err("AWS-LC initialization must reject conflict");
    let second =
        initialize_jwt_provider().expect_err("the initialization failure must be retained");
    assert!(first.is_provider_initialization());
    assert!(second.is_provider_initialization());

    let header = Header::new(Algorithm::RS256);
    assert!(
        encode_rsa_der(&header, &serde_json::json!({"sub": "test"}), b"not DER")
            .expect_err("DER signing must stop at the provider guard")
            .is_provider_initialization()
    );
    assert!(
        encode_rsa(&header, &serde_json::json!({"sub": "test"}), &private_key,)
            .expect_err("opaque-key signing must stop at the provider guard")
            .is_provider_initialization()
    );
    assert!(
        decode_rsa_der::<serde_json::Value>(
            "not.a.jwt",
            b"not DER",
            &Validation::new(Algorithm::RS256),
        )
        .expect_err("verification must stop at the provider guard")
        .is_provider_initialization()
    );
    assert!(
        rsa_public_jwk_from_private_der(b"not DER", Algorithm::RS256)
            .expect_err("DER JWK extraction must stop at the provider guard")
            .is_provider_initialization()
    );
    assert!(
        rsa_public_jwk(&private_key, Algorithm::RS256)
            .expect_err("opaque-key JWK extraction must stop at the provider guard")
            .is_provider_initialization()
    );

    let jwk: ProviderJwk = serde_json::from_value(serde_json::json!({
        "kty": "RSA",
        "alg": "RS256",
        "n": "AQAB",
        "e": "AQAB"
    }))
    .expect("parse sentinel JWK");
    assert!(
        decoding_key_from_jwk(&jwk)
            .expect_err("JWK key construction must stop at the provider guard")
            .is_provider_initialization()
    );
}
