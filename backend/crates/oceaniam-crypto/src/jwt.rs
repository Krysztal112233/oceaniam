use std::{error::Error, fmt, sync::OnceLock};

use serde::{Serialize, de::DeserializeOwned};

pub use jsonwebtoken::jwk::{Jwk as ProviderJwk, JwkSet as ProviderJwkSet};
pub use jsonwebtoken::{Algorithm, Header, TokenData, Validation};

#[derive(Debug, Clone, Copy)]
struct ProviderInitializationError;

static PROVIDER_INITIALIZATION: OnceLock<Result<(), ProviderInitializationError>> = OnceLock::new();

/// An error returned by an OceanIAM JWT mechanism.
#[derive(Debug)]
pub struct JwtError {
    kind: JwtErrorKind,
}

#[derive(Debug)]
enum JwtErrorKind {
    ProviderInitialization,
    Operation(jsonwebtoken::errors::Error),
}

impl JwtError {
    fn provider_initialization() -> Self {
        Self {
            kind: JwtErrorKind::ProviderInitialization,
        }
    }

    /// Creates the provider error used when a caller cannot export an RSA key as PKCS#1 DER.
    pub fn invalid_key_format() -> Self {
        jsonwebtoken::errors::new_error(jsonwebtoken::errors::ErrorKind::InvalidKeyFormat).into()
    }

    /// Returns whether the configured AWS-LC provider could not be installed.
    pub fn is_provider_initialization(&self) -> bool {
        matches!(self.kind, JwtErrorKind::ProviderInitialization)
    }
}

impl fmt::Display for JwtError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            JwtErrorKind::ProviderInitialization => {
                formatter.write_str("AWS-LC JWT provider initialization failed")
            }
            JwtErrorKind::Operation(source) => source.fmt(formatter),
        }
    }
}

impl Error for JwtError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.kind {
            JwtErrorKind::ProviderInitialization => None,
            JwtErrorKind::Operation(source) => Some(source),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for JwtError {
    fn from(source: jsonwebtoken::errors::Error) -> Self {
        Self {
            kind: JwtErrorKind::Operation(source),
        }
    }
}

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

/// Signs serializable claims with a PKCS#1-encoded RSA private key.
pub fn encode_rsa_der<T: Serialize>(
    header: &Header,
    claims: &T,
    private_key_der: &[u8],
) -> Result<String, JwtError> {
    initialize_jwt_provider()?;
    let key = jsonwebtoken::EncodingKey::from_rsa_der(private_key_der);
    jsonwebtoken::encode(header, claims, &key).map_err(Into::into)
}

/// Verifies and decodes claims with a PKCS#1-encoded RSA public key.
pub fn decode_rsa_der<T: DeserializeOwned>(
    token: impl AsRef<[u8]>,
    public_key_der: &[u8],
    validation: &Validation,
) -> Result<TokenData<T>, JwtError> {
    initialize_jwt_provider()?;
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

/// Extracts an RSA public JWK from a PKCS#1-encoded RSA private key.
pub fn rsa_public_jwk_from_private_der(
    private_key_der: &[u8],
    algorithm: Algorithm,
) -> Result<ProviderJwk, JwtError> {
    initialize_jwt_provider()?;
    let key = jsonwebtoken::EncodingKey::from_rsa_der(private_key_der);
    ProviderJwk::from_encoding_key(&key, algorithm).map_err(Into::into)
}
