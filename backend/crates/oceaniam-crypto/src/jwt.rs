use std::sync::OnceLock;

use aws_lc_rs::signature::{
    RSA_PKCS1_SHA256, RSA_PKCS1_SHA384, RSA_PKCS1_SHA512, RSA_PSS_SHA256, RSA_PSS_SHA384,
    RSA_PSS_SHA512, RsaEncoding,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::jwk::{
    AlgorithmParameters, CommonParameters, KeyAlgorithm, RSAKeyParameters, RSAKeyType,
};
use serde::{Serialize, de::DeserializeOwned};

use crate::RsaPrivateKey;
use crate::error::JwtError;
use crate::rsa::validate_der_document;

pub use jsonwebtoken::jwk::{Jwk as ProviderJwk, JwkSet as ProviderJwkSet};
pub use jsonwebtoken::{Algorithm, Header, TokenData, Validation};

#[derive(Debug, Clone, Copy)]
struct ProviderInitializationError;

static PROVIDER_INITIALIZATION: OnceLock<Result<(), ProviderInitializationError>> = OnceLock::new();

/// Installs jsonwebtoken's AWS-LC provider for this process.
///
/// The first result is retained for the lifetime of the process. If another provider was already
/// installed, this function and all provider-dependent operations fail closed.
pub fn initialize_jwt_provider() -> Result<(), JwtError> {
    match *PROVIDER_INITIALIZATION.get_or_init(|| {
        jsonwebtoken::crypto::aws_lc::DEFAULT_PROVIDER
            .install_default()
            .map_err(|_| ProviderInitializationError)
    }) {
        Ok(()) => Ok(()),
        Err(ProviderInitializationError) => Err(JwtError::provider_initialization()),
    }
}

/// An opaque JWT verification key constructed by this provider boundary.
#[derive(Clone, Debug)]
pub struct DecodingKey(jsonwebtoken::DecodingKey);

/// Parses a JWT header without validating its signature or claims.
pub fn decode_header(token: impl AsRef<[u8]>) -> Result<Header, JwtError> {
    jsonwebtoken::decode_header(token).map_err(Into::into)
}

/// Signs serializable claims with an opaque AWS-LC RSA private key.
pub fn encode_rsa<T: Serialize>(
    header: &Header,
    claims: &T,
    private_key: &RsaPrivateKey,
) -> Result<String, JwtError> {
    initialize_jwt_provider()?;
    let encoding = rsa_signing_encoding(header.alg)?;
    let encoded_header = encode_json_part(header)?;
    let encoded_claims = encode_json_part(claims)?;
    let message = format!("{encoded_header}.{encoded_claims}");
    let signature = private_key.sign(encoding, message.as_bytes())?;
    Ok(format!("{message}.{}", URL_SAFE_NO_PAD.encode(signature)))
}

/// Signs serializable claims with a PKCS#1-encoded RSA private key.
pub fn encode_rsa_der<T: Serialize>(
    header: &Header,
    claims: &T,
    private_key_der: &[u8],
) -> Result<String, JwtError> {
    initialize_jwt_provider()?;
    let private_key = RsaPrivateKey::from_pkcs1_der(private_key_der)?;
    encode_rsa(header, claims, &private_key)
}

/// Verifies and decodes claims with a PKCS#1-encoded RSA public key.
pub fn decode_rsa_der<T: DeserializeOwned>(
    token: impl AsRef<[u8]>,
    public_key_der: &[u8],
    validation: &Validation,
) -> Result<TokenData<T>, JwtError> {
    initialize_jwt_provider()?;
    validate_der_document(public_key_der)?;
    let key = jsonwebtoken::DecodingKey::from_rsa_der(public_key_der);
    jsonwebtoken::decode(token, &key, validation).map_err(Into::into)
}

/// Builds an opaque verification key from a provider JWK.
pub fn decoding_key_from_jwk(jwk: &ProviderJwk) -> Result<DecodingKey, JwtError> {
    initialize_jwt_provider()?;
    jsonwebtoken::DecodingKey::from_jwk(jwk)
        .map(DecodingKey)
        .map_err(Into::into)
}

/// Verifies and decodes claims with a provider-bound verification key.
pub fn decode<T: DeserializeOwned>(
    token: impl AsRef<[u8]>,
    key: &DecodingKey,
    validation: &Validation,
) -> Result<TokenData<T>, JwtError> {
    initialize_jwt_provider()?;
    jsonwebtoken::decode(token, &key.0, validation).map_err(Into::into)
}

/// Extracts an RSA public JWK from an opaque AWS-LC private key.
pub fn rsa_public_jwk(
    private_key: &RsaPrivateKey,
    algorithm: Algorithm,
) -> Result<ProviderJwk, JwtError> {
    initialize_jwt_provider()?;
    let key_algorithm = rsa_key_algorithm(algorithm)?;
    let components = private_key.public_key_components();

    Ok(ProviderJwk {
        common: CommonParameters {
            key_algorithm: Some(key_algorithm),
            ..Default::default()
        },
        algorithm: AlgorithmParameters::RSA(RSAKeyParameters {
            key_type: RSAKeyType::RSA,
            n: URL_SAFE_NO_PAD.encode(components.modulus()),
            e: URL_SAFE_NO_PAD.encode(components.exponent()),
        }),
    })
}

/// Extracts an RSA public JWK from a PKCS#1-encoded RSA private key.
pub fn rsa_public_jwk_from_private_der(
    private_key_der: &[u8],
    algorithm: Algorithm,
) -> Result<ProviderJwk, JwtError> {
    initialize_jwt_provider()?;
    let private_key = RsaPrivateKey::from_pkcs1_der(private_key_der)?;
    rsa_public_jwk(&private_key, algorithm)
}

fn encode_json_part<T: Serialize>(value: &T) -> Result<String, JwtError> {
    let json = serde_json::to_vec(value)
        .map_err(jsonwebtoken::errors::Error::from)
        .map_err(JwtError::from)?;
    Ok(URL_SAFE_NO_PAD.encode(json))
}

fn rsa_signing_encoding(algorithm: Algorithm) -> Result<&'static dyn RsaEncoding, JwtError> {
    match algorithm {
        Algorithm::RS256 => Ok(&RSA_PKCS1_SHA256),
        Algorithm::RS384 => Ok(&RSA_PKCS1_SHA384),
        Algorithm::RS512 => Ok(&RSA_PKCS1_SHA512),
        Algorithm::PS256 => Ok(&RSA_PSS_SHA256),
        Algorithm::PS384 => Ok(&RSA_PSS_SHA384),
        Algorithm::PS512 => Ok(&RSA_PSS_SHA512),
        _ => Err(jsonwebtoken::errors::new_error(
            jsonwebtoken::errors::ErrorKind::InvalidAlgorithm,
        )
        .into()),
    }
}

fn rsa_key_algorithm(algorithm: Algorithm) -> Result<KeyAlgorithm, JwtError> {
    match algorithm {
        Algorithm::RS256 => Ok(KeyAlgorithm::RS256),
        Algorithm::RS384 => Ok(KeyAlgorithm::RS384),
        Algorithm::RS512 => Ok(KeyAlgorithm::RS512),
        Algorithm::PS256 => Ok(KeyAlgorithm::PS256),
        Algorithm::PS384 => Ok(KeyAlgorithm::PS384),
        Algorithm::PS512 => Ok(KeyAlgorithm::PS512),
        _ => Err(jsonwebtoken::errors::new_error(
            jsonwebtoken::errors::ErrorKind::InvalidAlgorithm,
        )
        .into()),
    }
}
