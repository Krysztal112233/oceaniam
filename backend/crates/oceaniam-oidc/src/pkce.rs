//! Pure PKCE S256 verifier validation and verification.
//!
//! This module has no HTTP semantics and does not perform token, client, redirect URI, expiry, or
//! single-use checks. With S256, the compared value is a SHA-256 digest that an attacker can
//! compute offline for any candidate verifier, so comparison timing provides negligible advantage.
//! A constant-time comparison is nevertheless cheap defense in depth and consistent with the rest
//! of the backend.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use crate::error::PkceS256Error;

const CODE_VERIFIER_MIN_LENGTH: usize = 43;
const CODE_VERIFIER_MAX_LENGTH: usize = 128;
const S256_DIGEST_LENGTH: usize = 32;

/// Verifies an RFC 7636 S256 code verifier against an expected code challenge.
pub fn verify_code_verifier_s256(
    code_verifier: &str,
    code_challenge: &str,
) -> Result<(), PkceS256Error> {
    if !is_valid_code_verifier(code_verifier) {
        return Err(PkceS256Error::InvalidCodeVerifier);
    }

    let code_challenge =
        decode_s256_code_challenge(code_challenge).ok_or(PkceS256Error::ChallengeMismatch)?;
    let digest = Sha256::digest(code_verifier.as_bytes());

    if bool::from(digest.as_slice().ct_eq(code_challenge.as_slice())) {
        Ok(())
    } else {
        Err(PkceS256Error::ChallengeMismatch)
    }
}

pub(crate) fn decode_s256_code_challenge(value: &str) -> Option<[u8; S256_DIGEST_LENGTH]> {
    URL_SAFE_NO_PAD.decode(value).ok()?.try_into().ok()
}

fn is_valid_code_verifier(code_verifier: &str) -> bool {
    (CODE_VERIFIER_MIN_LENGTH..=CODE_VERIFIER_MAX_LENGTH).contains(&code_verifier.len())
        && code_verifier
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~'))
}

#[cfg(test)]
mod tests {
    use super::*;

    const RFC_7636_APPENDIX_B_VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    const RFC_7636_APPENDIX_B_CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

    fn challenge_for(code_verifier: &str) -> String {
        URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()))
    }

    // NOTE: AI-generated test
    #[test]
    fn verifies_rfc_7636_appendix_b_vector() {
        let decoded_challenge = decode_s256_code_challenge(RFC_7636_APPENDIX_B_CHALLENGE)
            .expect("RFC challenge should be canonical Base64URL");
        let digest = Sha256::digest(RFC_7636_APPENDIX_B_VERIFIER.as_bytes());

        assert_eq!(decoded_challenge.as_slice(), digest.as_slice());
        assert_eq!(
            challenge_for(RFC_7636_APPENDIX_B_VERIFIER),
            RFC_7636_APPENDIX_B_CHALLENGE
        );
        assert_eq!(
            verify_code_verifier_s256(RFC_7636_APPENDIX_B_VERIFIER, RFC_7636_APPENDIX_B_CHALLENGE),
            Ok(())
        );
    }

    // NOTE: AI-generated test
    #[test]
    fn enforces_inclusive_code_verifier_length_boundaries() {
        let minimum = "A".repeat(43);
        let maximum = format!("{}.~", "A".repeat(126));

        for verifier in [&minimum, &maximum] {
            assert_eq!(
                verify_code_verifier_s256(verifier, &challenge_for(verifier)),
                Ok(()),
                "length {} should be accepted",
                verifier.len()
            );
        }

        for verifier in ["A".repeat(42), "A".repeat(129)] {
            assert_eq!(
                verify_code_verifier_s256(&verifier, &challenge_for(&verifier)),
                Err(PkceS256Error::InvalidCodeVerifier),
                "length {} should be rejected",
                verifier.len()
            );
        }
    }

    // NOTE: AI-generated test
    #[test]
    fn accepts_only_ascii_unreserved_code_verifier_characters() {
        for &accepted in b"-._~" {
            for position in [0, 21, 42] {
                let mut bytes = vec![b'A'; 43];
                bytes[position] = accepted;
                let verifier = String::from_utf8(bytes).expect("test verifier should be ASCII");

                assert_eq!(
                    verify_code_verifier_s256(&verifier, &challenge_for(&verifier)),
                    Ok(()),
                    "character {} at position {position} should be accepted",
                    char::from(accepted)
                );
            }
        }

        let invalid_verifiers = [
            ("equals", format!("{}=", "A".repeat(42))),
            ("plus", format!("{}+", "A".repeat(42))),
            ("slash", format!("{}/", "A".repeat(42))),
            ("percent", format!("{}%", "A".repeat(42))),
            ("space", format!("{} ", "A".repeat(42))),
            ("control", format!("{}\n", "A".repeat(42))),
            ("non-ASCII letter", format!("{}é", "A".repeat(42))),
            ("emoji", format!("{}🙂", "A".repeat(42))),
        ];

        for (name, verifier) in invalid_verifiers {
            assert_eq!(
                verify_code_verifier_s256(&verifier, &challenge_for(&verifier)),
                Err(PkceS256Error::InvalidCodeVerifier),
                "case: {name}"
            );
        }
    }

    // NOTE: AI-generated test
    #[test]
    fn rejects_well_formed_verifiers_that_do_not_match_exactly() {
        let mismatches = [
            format!("A{}", &RFC_7636_APPENDIX_B_CHALLENGE[1..]),
            format!(
                "{}A",
                &RFC_7636_APPENDIX_B_CHALLENGE[..RFC_7636_APPENDIX_B_CHALLENGE.len() - 1]
            ),
            format!("e{}", &RFC_7636_APPENDIX_B_CHALLENGE[1..]),
            "A".repeat(43),
        ];

        for challenge in mismatches {
            assert_eq!(
                verify_code_verifier_s256(RFC_7636_APPENDIX_B_VERIFIER, &challenge),
                Err(PkceS256Error::ChallengeMismatch),
                "mismatch should fail"
            );
        }
    }

    // NOTE: AI-generated test
    #[test]
    fn maps_malformed_expected_challenges_to_mismatch() {
        let malformed_challenges = [
            format!("{}=", "A".repeat(42)),
            "A".repeat(42),
            "A".repeat(44),
            format!("{}B", "A".repeat(42)),
            format!("{}.", "A".repeat(42)),
            format!("{}+", "A".repeat(42)),
            format!("{}/", "A".repeat(42)),
            String::new(),
        ];

        for challenge in malformed_challenges {
            assert_eq!(
                verify_code_verifier_s256(RFC_7636_APPENDIX_B_VERIFIER, &challenge),
                Err(PkceS256Error::ChallengeMismatch)
            );
        }
    }
}
