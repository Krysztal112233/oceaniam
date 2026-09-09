use std::{sync::Arc, thread};

use base64::{
    Engine as _,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use oceaniam_crypto::{
    Algorithm, Header, RsaPrivateKey, Validation, decode_rsa_der, encode_rsa, rsa_public_jwk,
};
use serde_json::Value;
use zeroize::Zeroizing;

// NOTE: AI-generated test
#[test]
fn legacy_pkcs8_material_exports_identical_pem_public_der_and_components() {
    let key = RsaPrivateKey::from_pkcs8_pem(include_bytes!(
        "fixtures/legacy-rust-crypto/private-key.pkcs8.pem"
    ))
    .expect("load legacy RustCrypto PKCS#8 PEM with AWS-LC");
    let exported = key.to_pkcs8_pem().expect("export AWS-LC PKCS#8 PEM");
    assert_eq!(
        exported.as_bytes(),
        include_bytes!("fixtures/legacy-rust-crypto/private-key.pkcs8.pem")
    );
    assert_eq!(
        key.public_key_der(),
        include_bytes!("fixtures/legacy-rust-crypto/public-key.pkcs1.der")
    );

    let expected: Value =
        serde_json::from_str(include_str!("fixtures/legacy-rust-crypto/public-jwk.json"))
            .expect("parse expected JWK");
    let components = key.public_key_components();
    assert_eq!(
        URL_SAFE_NO_PAD.encode(components.modulus()),
        expected["n"].as_str().expect("fixture modulus")
    );
    assert_eq!(
        URL_SAFE_NO_PAD.encode(components.exponent()),
        expected["e"].as_str().expect("fixture exponent")
    );

    let jwk = rsa_public_jwk(&key, Algorithm::PS512).expect("derive JWK from opaque key");
    let actual = serde_json::to_value(jwk).expect("serialize derived JWK");
    assert_eq!(actual["n"], expected["n"]);
    assert_eq!(actual["e"], expected["e"]);
    assert_eq!(actual["alg"], "PS512");
}

// NOTE: AI-generated test
#[test]
fn pkcs8_pem_loader_accepts_crlf_and_surrounding_whitespace() {
    let fixture = include_bytes!("fixtures/legacy-rust-crypto/private-key.pkcs8.pem");
    let mut compatible = Zeroizing::new(Vec::with_capacity(fixture.len() + 64));
    compatible.extend_from_slice(b" \t\r\n");
    for byte in fixture {
        if *byte == b'\n' {
            compatible.extend_from_slice(b"\r\n");
        } else {
            compatible.push(*byte);
        }
    }
    compatible.extend_from_slice(b"\t \r\n");

    let key = RsaPrivateKey::from_pkcs8_pem(&compatible)
        .expect("compatible whitespace and CRLF should load");
    assert_eq!(key.bit_size(), 2048);
}

// NOTE: AI-generated test
#[test]
fn private_key_loaders_reject_invalid_pem_and_trailing_der() {
    for malformed in [
        b"-----BEGIN RSA PRIVATE KEY-----\nMAA=\n-----END RSA PRIVATE KEY-----\n".as_slice(),
        b"-----BEGIN PRIVATE KEY-----\n***\n-----END PRIVATE KEY-----\n".as_slice(),
        b"-----BEGIN PRIVATE KEY-----\n\n-----END PRIVATE KEY-----\n".as_slice(),
        b"-----BEGIN PRIVATE KEY-----\nProc-Type: 4,ENCRYPTED\nMAA=\n-----END PRIVATE KEY-----\n"
            .as_slice(),
        b"-----BEGIN PRIVATE KEY-----\nMAA=\n-----END PRIVATE KEY-----\n".as_slice(),
        b"prefix-----BEGIN PRIVATE KEY-----\nMAA=\n-----END PRIVATE KEY-----\n".as_slice(),
        b"-----BEGIN PRIVATE KEY-----\nMAA=\n-----END PRIVATE KEY-----suffix".as_slice(),
        b"-----BEGIN PRIVATE KEY-----\nMAA=\n-----END PUBLIC KEY-----\n".as_slice(),
    ] {
        assert!(RsaPrivateKey::from_pkcs8_pem(malformed).is_err());
    }

    let mut multiple = Zeroizing::new(Vec::new());
    let fixture = include_bytes!("fixtures/legacy-rust-crypto/private-key.pkcs8.pem");
    multiple.extend_from_slice(fixture);
    multiple.extend_from_slice(fixture);
    assert!(RsaPrivateKey::from_pkcs8_pem(&multiple).is_err());

    let fixture_text = std::str::from_utf8(fixture).expect("fixture PEM is UTF-8");
    let mut encoded_der = Zeroizing::new(String::with_capacity(fixture.len()));
    for line in fixture_text
        .lines()
        .filter(|line| !line.starts_with("-----"))
    {
        encoded_der.push_str(line);
    }
    let mut pkcs8 = Zeroizing::new(Vec::new());
    STANDARD
        .decode_vec(encoded_der.as_bytes(), &mut pkcs8)
        .expect("decode fixture PKCS#8 DER");
    pkcs8.extend_from_slice(b"junk");
    let mut trailing_der = Zeroizing::new(String::new());
    STANDARD.encode_string(&pkcs8, &mut trailing_der);
    let mut trailing_pem = Zeroizing::new(String::with_capacity(trailing_der.len() + 64));
    trailing_pem.push_str("-----BEGIN PRIVATE KEY-----\n");
    trailing_pem.push_str(&trailing_der);
    trailing_pem.push_str("\n-----END PRIVATE KEY-----\n");
    assert!(RsaPrivateKey::from_pkcs8_pem(trailing_pem.as_bytes()).is_err());
}

// NOTE: AI-generated test
#[test]
fn unsupported_generation_sizes_return_controlled_errors() {
    for bit_size in [0, 1024, 2056, 16_384] {
        let error = RsaPrivateKey::generate(bit_size)
            .expect_err("unsupported size must not generate a fallback key");
        assert!(error.is_unsupported_key_size());
        assert!(error.to_string().contains(&bit_size.to_string()));
    }
}

// NOTE: AI-generated test
#[test]
fn opaque_private_key_debug_output_contains_no_private_encoding() {
    let key = RsaPrivateKey::from_pkcs8_pem(include_bytes!(
        "fixtures/legacy-rust-crypto/private-key.pkcs8.pem"
    ))
    .expect("load fixture");
    let debug = format!("{key:?}");
    let pem = key.to_pkcs8_pem().expect("export fixture PEM");
    let pem_debug = format!("{pem:?}");

    assert_eq!(debug, "RsaPrivateKey { bit_size: 2048, .. }");
    assert!(!debug.contains("BEGIN PRIVATE KEY"));
    assert_eq!(pem_debug, "PrivateKeyPem(REDACTED)");
    assert!(!pem_debug.contains("BEGIN PRIVATE KEY"));
}

fn assert_clone_send_sync<T: Clone + Send + Sync>() {}

// NOTE: AI-generated test
#[test]
fn shared_private_key_clones_sign_concurrently_after_original_is_dropped() {
    assert_clone_send_sync::<RsaPrivateKey>();

    let original = RsaPrivateKey::from_pkcs8_pem(include_bytes!(
        "fixtures/legacy-rust-crypto/private-key.pkcs8.pem"
    ))
    .expect("load fixture");
    let public_der = Arc::new(original.public_key_der());
    let clones = (0..8).map(|_| original.clone()).collect::<Vec<_>>();
    drop(original);

    let threads = clones
        .into_iter()
        .enumerate()
        .map(|(index, private_key)| {
            let public_der = public_der.clone();
            thread::spawn(move || {
                let claims = serde_json::json!({
                    "sub": format!("clone-{index}"),
                    "exp": 4_102_444_800_u64,
                });
                let token = encode_rsa(&Header::new(Algorithm::RS256), &claims, &private_key)
                    .expect("concurrent signing should succeed");
                let decoded = decode_rsa_der::<Value>(
                    &token,
                    public_der.as_slice(),
                    &Validation::new(Algorithm::RS256),
                )
                .expect("concurrent signature should verify");
                assert_eq!(decoded.claims, claims);
            })
        })
        .collect::<Vec<_>>();

    for thread in threads {
        thread.join().expect("signing thread should not panic");
    }
}
