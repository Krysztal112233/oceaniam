use sha2::{Digest, Sha256};

const BROWSER_BINDING_PURPOSE: &[u8] = b"oceaniam:oidc:authorization:browser-binding:v1";
const CSRF_PURPOSE: &[u8] = b"oceaniam:oidc:authorization:csrf:v1";

/// Derives the persisted browser-binding digest from a per-transaction secret.
pub fn authorization_browser_binding_digest(secret: &[u8; 32]) -> [u8; 32] {
    purpose_separated_digest(BROWSER_BINDING_PURPOSE, secret)
}

/// Derives the persisted form-CSRF digest from a separate per-transaction secret.
pub fn authorization_csrf_digest(secret: &[u8; 32]) -> [u8; 32] {
    purpose_separated_digest(CSRF_PURPOSE, secret)
}

fn purpose_separated_digest(purpose: &[u8], secret: &[u8; 32]) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(purpose);
    digest.update([0]);
    digest.update(secret);
    digest.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: AI-generated test
    #[test]
    fn authorization_secret_digests_are_deterministic_and_purpose_separated() {
        let secret = [0x5a; 32];

        assert_eq!(
            authorization_browser_binding_digest(&secret),
            authorization_browser_binding_digest(&secret)
        );
        assert_ne!(
            authorization_browser_binding_digest(&secret),
            authorization_csrf_digest(&secret)
        );
        assert_ne!(authorization_browser_binding_digest(&secret), secret);
        assert_ne!(authorization_csrf_digest(&secret), secret);
    }
}
