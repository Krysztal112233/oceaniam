use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::{RngCore, rngs::OsRng};
use sha2::{Digest, Sha256};

const CODE_PURPOSE: &[u8] = b"oceaniam:oidc:authorization:code:v1";

/// Derives the persisted authorization-code digest from a plaintext code.
///
/// Only this purpose-separated SHA-256 digest is ever persisted; the plaintext code crosses the
/// 303 redirect to the client and is never stored or logged.
pub fn authorization_code_digest(code: &str) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(CODE_PURPOSE);
    digest.update([0]);
    digest.update(code.as_bytes());
    digest.finalize().into()
}

/// Generates a plaintext authorization code and its persisted digest.
///
/// The plaintext is 16 random bytes base64url-encoded without padding (22 characters), so it
/// needs no further encoding inside the redirect query. The digest is the only persistable
/// form; the plaintext must be dropped after the redirect is assembled.
pub fn generate_authorization_code() -> (String, [u8; 32]) {
    let mut secret = [0_u8; 16];
    OsRng.fill_bytes(&mut secret);
    let code = URL_SAFE_NO_PAD.encode(secret);
    (code.clone(), authorization_code_digest(&code))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authorization_browser_binding_digest;

    // NOTE: AI-generated test
    #[test]
    fn generated_codes_are_22_char_base64url_with_purpose_separated_digests() {
        let (code, digest) = generate_authorization_code();

        assert_eq!(code.len(), 22);
        assert!(
            code.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
            "code must be base64url URL_SAFE_NO_PAD: {code}"
        );
        assert_eq!(authorization_code_digest(&code), digest);

        let (other_code, other_digest) = generate_authorization_code();
        assert_ne!(code, other_code);
        assert_ne!(digest, other_digest);

        // Purpose separation: the same bytes under a different purpose digest differently.
        let mut secret = [0_u8; 32];
        secret[..16].copy_from_slice(&URL_SAFE_NO_PAD.decode(&code).expect("code decodes")[..]);
        assert_ne!(
            authorization_code_digest(&code),
            authorization_browser_binding_digest(&secret)
        );
    }
}
