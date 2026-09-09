#![deny(missing_docs)]

//! Provider-controlled cryptographic mechanisms used by OceanIAM.
//!
//! This crate owns the process-wide JWT provider and low-level JWT operations. Claim policy,
//! token lifetimes, key persistence, and application or tenant semantics belong to callers.
//! AWS-LC accepts RSA verification keys from 2048 through 8192 bits; unsupported keys fail without
//! a provider fallback.

mod error;
mod jwt;
mod rsa;

pub use error::{JwtError, RsaError};
pub use jwt::{
    Algorithm, DecodingKey, Header, ProviderJwk, ProviderJwkSet, TokenData, Validation, decode,
    decode_header, decode_rsa_der, decoding_key_from_jwk, encode_rsa, encode_rsa_der,
    initialize_jwt_provider, rsa_public_jwk, rsa_public_jwk_from_private_der,
};
pub use rsa::{PrivateKeyPem, RsaPrivateKey, RsaPublicKeyComponents};
