use aws_lc_rs::error::{KeyRejected, Unspecified};
use snafu::Snafu;

/// An error returned by an OceanIAM JWT mechanism.
#[derive(Debug, Snafu)]
#[snafu(transparent)]
pub struct JwtError {
    source: JwtErrorKind,
}

#[derive(Debug, Snafu)]
#[snafu(module(jwt_error_kind))]
enum JwtErrorKind {
    #[snafu(display("AWS-LC JWT provider initialization failed"))]
    ProviderInitialization,

    #[snafu(display("{source}"))]
    Operation { source: jsonwebtoken::errors::Error },

    #[snafu(display("{source}"))]
    Rsa { source: RsaError },
}

impl JwtError {
    pub(crate) fn provider_initialization() -> Self {
        Self {
            source: JwtErrorKind::ProviderInitialization,
        }
    }

    /// Creates the provider error used when a caller cannot export an RSA key as PKCS#1 DER.
    pub fn invalid_key_format() -> Self {
        jsonwebtoken::errors::new_error(jsonwebtoken::errors::ErrorKind::InvalidKeyFormat).into()
    }

    /// Returns whether the configured AWS-LC provider could not be installed.
    pub fn is_provider_initialization(&self) -> bool {
        matches!(self.source, JwtErrorKind::ProviderInitialization)
    }
}

impl From<jsonwebtoken::errors::Error> for JwtError {
    fn from(source: jsonwebtoken::errors::Error) -> Self {
        Self {
            source: JwtErrorKind::Operation { source },
        }
    }
}

impl From<RsaError> for JwtError {
    fn from(source: RsaError) -> Self {
        Self {
            source: JwtErrorKind::Rsa { source },
        }
    }
}

/// An error returned by the AWS-LC RSA material boundary.
#[derive(Debug, Snafu)]
#[snafu(transparent)]
pub struct RsaError {
    source: RsaErrorKind,
}

#[derive(Debug, Snafu)]
#[snafu(module(rsa_error_kind))]
enum RsaErrorKind {
    #[snafu(display("unsupported RSA key size: {bits} bits"))]
    UnsupportedKeySize { bits: usize },

    #[snafu(display("invalid RSA PKCS#8 PEM encoding"))]
    InvalidPem,

    #[snafu(display("invalid RSA DER encoding"))]
    InvalidDer,

    #[snafu(display("RSA private key encoding failed"))]
    Encoding,

    #[snafu(display("AWS-LC rejected RSA private key: {source}"))]
    KeyRejected { source: KeyRejected },

    #[snafu(display("AWS-LC RSA operation failed"))]
    Operation { source: Unspecified },
}

impl RsaError {
    pub(crate) fn unsupported_key_size(bits: usize) -> Self {
        Self {
            source: RsaErrorKind::UnsupportedKeySize { bits },
        }
    }

    pub(crate) fn invalid_pem() -> Self {
        Self {
            source: RsaErrorKind::InvalidPem,
        }
    }

    pub(crate) fn invalid_der() -> Self {
        Self {
            source: RsaErrorKind::InvalidDer,
        }
    }

    pub(crate) fn encoding() -> Self {
        Self {
            source: RsaErrorKind::Encoding,
        }
    }

    /// Returns whether key generation rejected an unsupported requested size.
    pub fn is_unsupported_key_size(&self) -> bool {
        matches!(self.source, RsaErrorKind::UnsupportedKeySize { .. })
    }
}

impl From<KeyRejected> for RsaError {
    fn from(source: KeyRejected) -> Self {
        Self {
            source: RsaErrorKind::KeyRejected { source },
        }
    }
}

impl From<Unspecified> for RsaError {
    fn from(source: Unspecified) -> Self {
        Self {
            source: RsaErrorKind::Operation { source },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error as _;

    use super::*;

    // NOTE: AI-generated test
    #[test]
    fn snafu_wrappers_preserve_the_public_error_contract() {
        let provider_error = JwtError::provider_initialization();
        assert_eq!(
            provider_error.to_string(),
            "AWS-LC JWT provider initialization failed"
        );
        assert!(provider_error.source().is_none());
        assert!(provider_error.is_provider_initialization());

        let operation_source =
            jsonwebtoken::errors::new_error(jsonwebtoken::errors::ErrorKind::InvalidKeyFormat);
        let operation_display = operation_source.to_string();
        let operation_error = JwtError::from(operation_source);
        assert_eq!(operation_error.to_string(), operation_display);
        assert_eq!(
            operation_error.source().map(ToString::to_string),
            Some(operation_display)
        );
        assert!(!operation_error.is_provider_initialization());

        let rsa_error = RsaError::unsupported_key_size(1_024);
        assert_eq!(rsa_error.to_string(), "unsupported RSA key size: 1024 bits");
        assert!(rsa_error.source().is_none());
        assert!(rsa_error.is_unsupported_key_size());
    }
}
