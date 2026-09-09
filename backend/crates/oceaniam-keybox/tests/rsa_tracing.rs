use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};

use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use oceaniam_auth::{
    Algorithm, Header, TokenData, Validation,
    jwt::{ClaimHelper, JwtCodec, SystemClaim},
};
use oceaniam_common::{consts, crypto::MasterKey};
use oceaniam_keybox::{KeyAlg, RawKey, RsaKey, SecretField};
use tracing::{Id, Instrument as _, Subscriber, span::Attributes};
use tracing_subscriber::{
    fmt::format::FmtSpan,
    layer::{Context, Layer},
    prelude::*,
};
use uuid::Uuid;

#[derive(Clone)]
struct Buffer(Arc<Mutex<Vec<u8>>>);

impl Write for Buffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .expect("trace buffer lock")
            .extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Clone)]
struct SpanRecorder(Arc<Mutex<Vec<&'static str>>>);

impl<S> Layer<S> for SpanRecorder
where
    S: Subscriber,
{
    fn on_new_span(&self, attributes: &Attributes<'_>, _id: &Id, _context: Context<'_, S>) {
        self.0
            .lock()
            .expect("span recorder lock")
            .push(attributes.metadata().name());
    }
}

// NOTE: AI-generated test
#[tokio::test]
async fn rsa_material_traces_preserve_context_without_leaking_secrets() {
    let bytes = Arc::new(Mutex::new(Vec::new()));
    let writer = bytes.clone();
    let span_names = Arc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::registry()
        .with(SpanRecorder(span_names.clone()))
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(move || Buffer(writer.clone()))
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE),
        );
    let dispatch = tracing::Dispatch::new(subscriber);
    let _default = tracing::dispatcher::set_default(&dispatch);
    let master_key =
        MasterKey::from_hex("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
            .expect("test master key");
    let algorithm = KeyAlg::try_from(Algorithm::RS512).expect("supported algorithm");
    let subject_id = Uuid::now_v7();

    let (token, recovered) = async {
        let key = RsaKey::generate(Uuid::now_v7(), algorithm.clone())
            .await
            .expect("generate test key");
        let raw = key.into_raw_key(&master_key).expect("seal generated key");
        let recovered = RsaKey::from_raw_key(raw, &master_key).expect("unseal generated key");
        let token = recovered
            .encode(
                Header::new(Algorithm::RS512),
                SystemClaim::new(subject_id, 60, None, None),
            )
            .expect("sign with restored key");
        let _: TokenData<SystemClaim> = recovered
            .decode(token.as_bytes(), &Validation::new(Algorithm::RS512))
            .expect("verify with restored key");
        (token, recovered)
    }
    .instrument(tracing::info_span!("test.request"))
    .await;

    const INVALID_PLAINTEXT: &[u8] = b"SENSITIVE-INVALID-PRIVATE-KEY-MARKER";
    let invalid_blob = master_key
        .encrypt(INVALID_PLAINTEXT, consts::KEK_VERSION_CURRENT)
        .expect("encrypt malformed private-key plaintext");
    let invalid_raw = RawKey {
        key_id: Uuid::now_v7(),
        key_alg: algorithm,
        secret: serde_json::to_value(SecretField {
            nonce: B64.encode(invalid_blob.nonce),
            ciphertext: B64.encode(invalid_blob.ciphertext),
            key_version: invalid_blob.key_version,
        })
        .expect("serialize malformed encrypted secret"),
    };
    RsaKey::from_raw_key(invalid_raw, &master_key)
        .expect_err("malformed decrypted PEM must fail closed");
    drop(recovered);

    let names = span_names.lock().expect("span recorder lock");
    for expected in [
        "test.request",
        "keybox.rsa.queue",
        "keybox.rsa.generate",
        "keybox.rsa.into_raw",
        "keybox.private_key.seal",
        "keybox.rsa.from_raw",
        "keybox.private_key.unseal",
        "auth.jwt.encode",
        "auth.jwt.decode",
    ] {
        assert!(
            names.contains(&expected),
            "expected tracing span {expected} in {names:?}"
        );
    }
    drop(names);

    let output = String::from_utf8(bytes.lock().expect("trace buffer lock").clone())
        .expect("UTF-8 trace output");
    assert!(!output.contains("BEGIN PRIVATE KEY"));
    assert!(!output.contains(&subject_id.to_string()));
    assert!(!output.contains(&token));
    assert!(!output.contains(std::str::from_utf8(INVALID_PLAINTEXT).expect("ASCII marker")));
}
