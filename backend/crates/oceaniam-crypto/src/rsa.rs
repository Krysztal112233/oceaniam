use std::{fmt, mem, sync::Arc};

use aws_lc_rs::{
    encoding::{AsDer, Pkcs8V1Der},
    rand::SystemRandom,
    rsa::{KeyPair, KeySize, PublicKeyComponents},
    signature::{KeyPair as _, RsaEncoding},
};
use base64::{Engine as _, encoded_len, engine::general_purpose::STANDARD};
use zeroize::Zeroizing;

use crate::error::RsaError;

const PEM_BEGIN: &[u8] = b"-----BEGIN PRIVATE KEY-----";
const PEM_END: &[u8] = b"-----END PRIVATE KEY-----";
const PEM_LINE_LENGTH: usize = 64;

/// A zeroizing PKCS#8 PEM document whose debug output is always redacted.
pub struct PrivateKeyPem(Zeroizing<String>);

impl PrivateKeyPem {
    /// Borrows the canonical PEM bytes.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl fmt::Debug for PrivateKeyPem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PrivateKeyPem(REDACTED)")
    }
}

/// Opaque AWS-LC-owned RSA private-key material.
#[derive(Clone)]
pub struct RsaPrivateKey {
    key_pair: Arc<KeyPair>,
}

impl RsaPrivateKey {
    /// Generates an RSA private key at an AWS-LC-supported size.
    pub fn generate(bit_size: usize) -> Result<Self, RsaError> {
        Ok(Self {
            key_pair: Arc::new(KeyPair::generate(key_size(bit_size)?)?),
        })
    }

    fn from_pkcs8_der(der: &[u8]) -> Result<Self, RsaError> {
        validate_der_document(der)?;
        Ok(Self {
            key_pair: Arc::new(KeyPair::from_pkcs8(der)?),
        })
    }

    pub(crate) fn from_pkcs1_der(der: &[u8]) -> Result<Self, RsaError> {
        validate_der_document(der)?;
        Ok(Self {
            key_pair: Arc::new(KeyPair::from_der(der)?),
        })
    }

    /// Strictly loads one unencrypted `PRIVATE KEY` PKCS#8 PEM document.
    ///
    /// Canonical LF input and CRLF input with surrounding ASCII whitespace are accepted. Prefixes,
    /// suffixes, headers, multiple documents, wrong labels, invalid base64, and trailing DER are
    /// rejected.
    pub fn from_pkcs8_pem(pem: &[u8]) -> Result<Self, RsaError> {
        let document = trim_ascii_whitespace(pem);
        let after_begin = document
            .strip_prefix(PEM_BEGIN)
            .and_then(strip_line_break_prefix)
            .ok_or_else(RsaError::invalid_pem)?;
        let before_end = after_begin
            .strip_suffix(PEM_END)
            .and_then(strip_line_break_suffix)
            .ok_or_else(RsaError::invalid_pem)?;

        let mut encoded = Zeroizing::new(String::with_capacity(before_end.len()));
        let mut line_length = 0;
        let mut index = 0;
        while index < before_end.len() {
            let byte = before_end[index];
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'+' | b'/' | b'=' => {
                    encoded.push(char::from(byte));
                    line_length += 1;
                }
                b'\n' if line_length != 0 => line_length = 0,
                b'\r' if line_length != 0 && before_end.get(index + 1).copied() == Some(b'\n') => {
                    line_length = 0;
                    index += 1;
                }
                _ => return Err(RsaError::invalid_pem()),
            }
            index += 1;
        }
        if encoded.is_empty() || line_length == 0 {
            return Err(RsaError::invalid_pem());
        }

        let mut der = Zeroizing::new(Vec::new());
        STANDARD
            .decode_vec(encoded.as_bytes(), &mut der)
            .map_err(|_| RsaError::invalid_pem())?;
        Self::from_pkcs8_der(&der)
    }

    /// Exports the private key as canonical LF-ended PKCS#8 `PRIVATE KEY` PEM.
    pub fn to_pkcs8_pem(&self) -> Result<PrivateKeyPem, RsaError> {
        let der: Pkcs8V1Der<'static> = self.key_pair.as_der()?;
        let encoded_capacity =
            encoded_len(der.as_ref().len(), true).ok_or_else(RsaError::encoding)?;
        let mut encoded = Zeroizing::new(vec![0; encoded_capacity]);
        let encoded_length = STANDARD
            .encode_slice(der.as_ref(), &mut encoded)
            .map_err(|_| RsaError::encoding())?;
        if encoded_length != encoded_capacity {
            return Err(RsaError::encoding());
        }

        let pem_capacity = encoded.len()
            + encoded.len().div_ceil(PEM_LINE_LENGTH)
            + PEM_BEGIN.len()
            + PEM_END.len()
            + 3;
        let mut pem = Zeroizing::new(String::with_capacity(pem_capacity));
        pem.push_str("-----BEGIN PRIVATE KEY-----\n");
        for line in encoded.chunks(PEM_LINE_LENGTH) {
            let line = std::str::from_utf8(line).map_err(|_| RsaError::invalid_pem())?;
            pem.push_str(line);
            pem.push('\n');
        }
        pem.push_str("-----END PRIVATE KEY-----\n");
        Ok(PrivateKeyPem(pem))
    }

    /// Returns the RFC 8017 PKCS#1 DER public key.
    pub fn public_key_der(&self) -> Vec<u8> {
        self.key_pair.public_key().as_ref().to_vec()
    }

    /// Returns the unsigned big-endian RSA public modulus and exponent.
    pub fn public_key_components(&self) -> RsaPublicKeyComponents {
        let components = PublicKeyComponents::<Vec<u8>>::from(self.key_pair.public_key());
        RsaPublicKeyComponents {
            modulus: components.n,
            exponent: components.e,
        }
    }

    /// Returns the exact RSA modulus length in bits.
    pub fn bit_size(&self) -> usize {
        modulus_bit_length(self.public_key_components().modulus())
    }

    pub(crate) fn sign(
        &self,
        encoding: &'static dyn RsaEncoding,
        message: &[u8],
    ) -> Result<Vec<u8>, RsaError> {
        let mut signature = vec![0; self.key_pair.public_modulus_len()];
        self.key_pair
            .sign(encoding, &SystemRandom::new(), message, &mut signature)?;
        Ok(signature)
    }
}

impl fmt::Debug for RsaPrivateKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RsaPrivateKey")
            .field("bit_size", &self.bit_size())
            .finish_non_exhaustive()
    }
}

impl PartialEq for RsaPrivateKey {
    fn eq(&self, other: &Self) -> bool {
        self.key_pair.public_key().as_ref() == other.key_pair.public_key().as_ref()
    }
}

impl Eq for RsaPrivateKey {}

/// Owned public components extracted from an RSA private key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RsaPublicKeyComponents {
    modulus: Vec<u8>,
    exponent: Vec<u8>,
}

impl RsaPublicKeyComponents {
    /// Returns the unsigned big-endian modulus.
    pub fn modulus(&self) -> &[u8] {
        &self.modulus
    }

    /// Returns the unsigned big-endian public exponent.
    pub fn exponent(&self) -> &[u8] {
        &self.exponent
    }
}

fn key_size(bit_size: usize) -> Result<KeySize, RsaError> {
    match bit_size {
        2048 => Ok(KeySize::Rsa2048),
        3072 => Ok(KeySize::Rsa3072),
        4096 => Ok(KeySize::Rsa4096),
        8192 => Ok(KeySize::Rsa8192),
        _ => Err(RsaError::unsupported_key_size(bit_size)),
    }
}

fn modulus_bit_length(modulus: &[u8]) -> usize {
    let Some((first_index, first_byte)) = modulus.iter().enumerate().find(|(_, byte)| **byte != 0)
    else {
        return 0;
    };

    (modulus.len() - first_index - 1) * 8 + (8 - first_byte.leading_zeros() as usize)
}

pub(crate) fn validate_der_document(der: &[u8]) -> Result<(), RsaError> {
    if der.first().copied() != Some(0x30) || der.len() < 2 {
        return Err(RsaError::invalid_der());
    }

    let first_length = der[1];
    let (content_length, header_length) = if first_length & 0x80 == 0 {
        (usize::from(first_length), 2)
    } else {
        let length_bytes = usize::from(first_length & 0x7f);
        if length_bytes == 0
            || length_bytes > mem::size_of::<usize>()
            || der.len() < 2 + length_bytes
            || der[2] == 0
        {
            return Err(RsaError::invalid_der());
        }

        let mut length = 0usize;
        for byte in &der[2..2 + length_bytes] {
            length = length
                .checked_mul(256)
                .and_then(|value| value.checked_add(usize::from(*byte)))
                .ok_or_else(RsaError::invalid_der)?;
        }
        if length < 128 {
            return Err(RsaError::invalid_der());
        }
        (length, 2 + length_bytes)
    };

    if header_length.checked_add(content_length) != Some(der.len()) {
        return Err(RsaError::invalid_der());
    }
    Ok(())
}

fn trim_ascii_whitespace(mut input: &[u8]) -> &[u8] {
    while input.first().is_some_and(u8::is_ascii_whitespace) {
        input = &input[1..];
    }
    while input.last().is_some_and(u8::is_ascii_whitespace) {
        input = &input[..input.len() - 1];
    }
    input
}

fn strip_line_break_prefix(input: &[u8]) -> Option<&[u8]> {
    input
        .strip_prefix(b"\r\n")
        .or_else(|| input.strip_prefix(b"\n"))
}

fn strip_line_break_suffix(input: &[u8]) -> Option<&[u8]> {
    input
        .strip_suffix(b"\r\n")
        .or_else(|| input.strip_suffix(b"\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: AI-generated test
    #[test]
    fn supported_key_sizes_map_without_generating_keys() {
        assert_eq!(key_size(2048).expect("2048-bit mapping"), KeySize::Rsa2048);
        assert_eq!(key_size(3072).expect("3072-bit mapping"), KeySize::Rsa3072);
        assert_eq!(key_size(4096).expect("4096-bit mapping"), KeySize::Rsa4096);
        assert_eq!(key_size(8192).expect("8192-bit mapping"), KeySize::Rsa8192);
        for unsupported in [0, 1024, 2049, 6144, usize::MAX] {
            assert!(
                key_size(unsupported)
                    .expect_err("unsupported size must fail")
                    .is_unsupported_key_size()
            );
        }
    }

    // NOTE: AI-generated test
    #[test]
    fn modulus_bit_length_does_not_round_odd_bit_lengths_up() {
        let mut modulus = vec![0; 257];
        modulus[0] = 1;
        assert_eq!(modulus_bit_length(&modulus), 2049);
        assert_eq!(modulus_bit_length(&[0, 0x80]), 8);
        assert_eq!(modulus_bit_length(&[0, 1]), 1);
        assert_eq!(modulus_bit_length(&[]), 0);
        assert_eq!(modulus_bit_length(&[0, 0]), 0);
    }

    // NOTE: AI-generated test
    #[test]
    fn der_document_validation_rejects_incomplete_or_noncanonical_lengths() {
        for invalid in [
            &[][..],
            &[0x30][..],
            &[0x31, 0x00][..],
            &[0x30, 0x80, 0x00, 0x00][..],
            &[0x30, 0x81, 0x00][..],
            &[0x30, 0x81, 0x01, 0x00][..],
            &[0x30, 0x82, 0x01][..],
            &[0x30, 0x00, 0x00][..],
        ] {
            assert!(validate_der_document(invalid).is_err());
        }

        assert!(validate_der_document(&[0x30, 0x00]).is_ok());
        assert!(validate_der_document(&[0x30, 0x01, 0x00]).is_ok());
        let mut long_form = vec![0x30, 0x81, 0x80];
        long_form.extend_from_slice(&[0; 128]);
        assert!(validate_der_document(&long_form).is_ok());
    }
}
