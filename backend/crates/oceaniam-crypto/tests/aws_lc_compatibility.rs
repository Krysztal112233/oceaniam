use std::{collections::BTreeMap, thread};

use oceaniam_crypto::{
    Algorithm, Header, ProviderJwk, Validation, decode, decode_header, decode_rsa_der,
    decoding_key_from_jwk, encode_rsa_der, initialize_jwt_provider,
    rsa_public_jwk_from_private_der,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const KEY_ID: &str = "018f47a5-f53c-7bd0-bad8-764ae9bac80b";
const ISSUER: &str = "https://legacy.oceaniam.test";
const AUDIENCE: &str = "legacy-client";

const ALGORITHMS: [(Algorithm, &str); 6] = [
    (Algorithm::RS256, "RS256"),
    (Algorithm::RS384, "RS384"),
    (Algorithm::RS512, "RS512"),
    (Algorithm::PS256, "PS256"),
    (Algorithm::PS384, "PS384"),
    (Algorithm::PS512, "PS512"),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct FixtureClaims {
    sub: String,
    exp: i64,
    iat: i64,
    iss: Option<String>,
    aud: Option<Vec<String>>,
    jti: String,
}

fn fixture_claims() -> FixtureClaims {
    FixtureClaims {
        sub: "018f47a6-7b53-7cc0-8f5c-5c8bf2d6b8cf".to_owned(),
        exp: 4_102_444_800,
        iat: 1_700_000_000,
        iss: Some(ISSUER.to_owned()),
        aud: Some(vec![AUDIENCE.to_owned()]),
        jti: "018f47a6-a340-75f0-94b9-03562a28a721".to_owned(),
    }
}

fn validation(algorithm: Algorithm) -> Validation {
    let mut validation = Validation::new(algorithm);
    validation.set_issuer(&[ISSUER]);
    validation.set_audience(&[AUDIENCE]);
    validation
}

fn altered_segment(token: &str, index: usize) -> String {
    let mut segments = token.split('.').map(str::to_owned).collect::<Vec<_>>();
    let mut bytes = segments[index].clone().into_bytes();
    bytes[0] = if bytes[0] == b'A' { b'B' } else { b'A' };
    segments[index] = String::from_utf8(bytes).expect("base64url remains UTF-8");
    segments.join(".")
}

// NOTE: AI-generated test
#[test]
fn initializer_is_idempotent_under_concurrent_calls() {
    let threads = (0..16)
        .map(|_| thread::spawn(initialize_jwt_provider))
        .collect::<Vec<_>>();

    for thread in threads {
        thread
            .join()
            .expect("initializer thread should not panic")
            .expect("AWS-LC provider should initialize");
    }
    initialize_jwt_provider().expect("repeated initialization should retain success");
}

// NOTE: AI-generated test
#[test]
fn aws_lc_verifies_legacy_tokens_and_round_trips_all_rsa_algorithms() {
    let private_der = include_bytes!("fixtures/legacy-rust-crypto/private-key.pkcs1.der");
    let public_der = include_bytes!("fixtures/legacy-rust-crypto/public-key.pkcs1.der");
    let legacy_tokens: BTreeMap<String, String> =
        serde_json::from_str(include_str!("fixtures/legacy-rust-crypto/tokens.json"))
            .expect("parse legacy token fixture");
    let expected_jwk: Value =
        serde_json::from_str(include_str!("fixtures/legacy-rust-crypto/public-jwk.json"))
            .expect("parse legacy JWK fixture");
    let claims = fixture_claims();

    for (algorithm, name) in ALGORITHMS {
        let legacy_token = &legacy_tokens[name];
        let validation = validation(algorithm);
        let mut jwk = rsa_public_jwk_from_private_der(private_der, algorithm)
            .expect("extract AWS-LC public JWK");
        jwk.common.key_id = Some(KEY_ID.to_owned());
        let actual_jwk = serde_json::to_value(&jwk).expect("serialize extracted JWK");

        assert_eq!(actual_jwk["kty"], expected_jwk["kty"]);
        assert_eq!(actual_jwk["n"], expected_jwk["n"]);
        assert_eq!(actual_jwk["e"], expected_jwk["e"]);
        assert_eq!(actual_jwk["kid"], expected_jwk["kid"]);
        assert_eq!(actual_jwk["alg"], name);
        assert!(actual_jwk.get("use").is_none());

        let legacy_der = decode_rsa_der::<FixtureClaims>(legacy_token, public_der, &validation)
            .expect("AWS-LC should verify the legacy token with public DER");
        assert_eq!(legacy_der.claims, claims);
        assert_eq!(legacy_der.header.alg, algorithm);
        assert_eq!(legacy_der.header.kid.as_deref(), Some(KEY_ID));

        let decoding_key = decoding_key_from_jwk(&jwk).expect("construct JWK verification key");
        let legacy_jwk = decode::<FixtureClaims>(legacy_token, &decoding_key, &validation)
            .expect("AWS-LC should verify the legacy token with its public JWK");
        assert_eq!(legacy_jwk.claims, claims);

        let mut header = Header::new(algorithm);
        header.kid = Some(KEY_ID.to_owned());
        let token = encode_rsa_der(&header, &claims, private_der)
            .expect("AWS-LC should sign with the legacy key");
        let legacy_segments = legacy_token.split('.').collect::<Vec<_>>();
        let new_segments = token.split('.').collect::<Vec<_>>();
        assert_eq!(&new_segments[..2], &legacy_segments[..2]);
        if matches!(
            algorithm,
            Algorithm::RS256 | Algorithm::RS384 | Algorithm::RS512
        ) {
            assert_eq!(&token, legacy_token);
        }

        assert_eq!(
            decode_rsa_der::<FixtureClaims>(&token, public_der, &validation)
                .expect("verify AWS-LC token with public DER")
                .claims,
            claims
        );
        assert_eq!(
            decode::<FixtureClaims>(&token, &decoding_key, &validation)
                .expect("verify AWS-LC token with public JWK")
                .claims,
            claims
        );
        let decoded_header = decode_header(&token).expect("decode signed header");
        assert_eq!(decoded_header.alg, algorithm);
        assert_eq!(decoded_header.kid.as_deref(), Some(KEY_ID));
        assert_eq!(decoded_header.typ.as_deref(), Some("JWT"));
    }
}

// NOTE: AI-generated test
#[test]
fn aws_lc_rejects_tampering_wrong_keys_algorithms_and_malformed_keys() {
    let private_der = include_bytes!("fixtures/legacy-rust-crypto/private-key.pkcs1.der");
    let public_der = include_bytes!("fixtures/legacy-rust-crypto/public-key.pkcs1.der");
    let unrelated_der =
        include_bytes!("fixtures/legacy-rust-crypto/unrelated-public-key.pkcs1.der");
    let legacy_tokens: BTreeMap<String, String> =
        serde_json::from_str(include_str!("fixtures/legacy-rust-crypto/tokens.json"))
            .expect("parse legacy token fixture");
    let token = &legacy_tokens["RS256"];

    assert!(
        decode_rsa_der::<FixtureClaims>(
            altered_segment(token, 1),
            public_der,
            &validation(Algorithm::RS256),
        )
        .is_err()
    );
    assert!(
        decode_rsa_der::<FixtureClaims>(
            altered_segment(token, 2),
            public_der,
            &validation(Algorithm::RS256),
        )
        .is_err()
    );
    assert!(
        decode_rsa_der::<FixtureClaims>(token, unrelated_der, &validation(Algorithm::RS256),)
            .is_err()
    );
    assert!(
        decode_rsa_der::<FixtureClaims>(token, public_der, &validation(Algorithm::RS384),).is_err()
    );

    let mut wrong_issuer = validation(Algorithm::RS256);
    wrong_issuer.set_issuer(&["https://wrong-issuer.oceaniam.test"]);
    assert!(decode_rsa_der::<FixtureClaims>(token, public_der, &wrong_issuer).is_err());
    let mut wrong_audience = validation(Algorithm::RS256);
    wrong_audience.set_audience(&["wrong-client"]);
    assert!(decode_rsa_der::<FixtureClaims>(token, public_der, &wrong_audience).is_err());

    let current = encode_rsa_der(
        &Header::new(Algorithm::RS256),
        &fixture_claims(),
        private_der,
    )
    .expect("sign unexpired control claims");
    assert!(
        decode_rsa_der::<FixtureClaims>(&current, public_der, &validation(Algorithm::RS256))
            .is_ok()
    );

    let mut expired_claims = fixture_claims();
    expired_claims.exp = 1;
    let expired = encode_rsa_der(&Header::new(Algorithm::RS256), &expired_claims, private_der)
        .expect("sign expired claims");
    assert!(
        decode_rsa_der::<FixtureClaims>(&expired, public_der, &validation(Algorithm::RS256))
            .is_err()
    );
    let missing_exp = encode_rsa_der(
        &Header::new(Algorithm::RS256),
        &serde_json::json!({"sub": "missing-exp"}),
        private_der,
    )
    .expect("sign claims without exp");
    assert!(
        decode_rsa_der::<Value>(&missing_exp, public_der, &Validation::new(Algorithm::RS256),)
            .is_err()
    );

    assert!(
        decode_rsa_der::<FixtureClaims>(token, b"not DER", &validation(Algorithm::RS256)).is_err()
    );
    assert!(
        encode_rsa_der(
            &Header::new(Algorithm::RS256),
            &fixture_claims(),
            b"not DER",
        )
        .is_err()
    );

    let malformed_jwk: ProviderJwk = serde_json::from_value(serde_json::json!({
        "kty": "RSA",
        "alg": "RS256",
        "n": "***",
        "e": "AQAB"
    }))
    .expect("deserialize structurally valid malformed JWK");
    assert!(decoding_key_from_jwk(&malformed_jwk).is_err());
}

// NOTE: AI-generated test
#[test]
fn aws_lc_rejects_a_valid_legacy_key_below_its_supported_size() {
    let token = include_str!("fixtures/legacy-rust-crypto/unsupported-1024-rs256.jwt").trim();
    let public_der =
        include_bytes!("fixtures/legacy-rust-crypto/unsupported-1024-public-key.pkcs1.der");
    let public_jwk: ProviderJwk = serde_json::from_str(include_str!(
        "fixtures/legacy-rust-crypto/unsupported-1024-public-jwk.json"
    ))
    .expect("parse valid legacy 1024-bit JWK");
    let validation = validation(Algorithm::RS256);

    assert!(decode_rsa_der::<FixtureClaims>(token, public_der, &validation).is_err());
    let decoding_key =
        decoding_key_from_jwk(&public_jwk).expect("construct key from valid JWK components");
    assert!(decode::<FixtureClaims>(token, &decoding_key, &validation).is_err());
}
