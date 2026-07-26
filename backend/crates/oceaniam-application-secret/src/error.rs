use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationSecretError {
    InvalidSecretFormat,
    InvalidKeyEncoding,
    InvalidKeyLength,
    AllZeroKey,
    InvalidKeyVersion(i32),
    DuplicateKeyVersion(i32),
    MissingCurrentKey(i32),
    MissingKeyVersion(i32),
    InvalidVerifierLength(usize),
}

impl fmt::Display for ApplicationSecretError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSecretFormat => formatter.write_str(
                "application secret must be `app_` followed by 32 ASCII alphanumeric characters",
            ),
            Self::InvalidKeyEncoding => formatter.write_str("HMAC key must be hexadecimal"),
            Self::InvalidKeyLength => formatter.write_str("HMAC key must contain exactly 32 bytes"),
            Self::AllZeroKey => formatter.write_str("HMAC key must not be all zeroes"),
            Self::InvalidKeyVersion(version) => {
                write!(
                    formatter,
                    "HMAC key version must be positive, got {version}"
                )
            }
            Self::DuplicateKeyVersion(version) => {
                write!(formatter, "duplicate HMAC key version {version}")
            }
            Self::MissingCurrentKey(version) => {
                write!(
                    formatter,
                    "current HMAC key version {version} is not configured"
                )
            }
            Self::MissingKeyVersion(version) => {
                write!(formatter, "HMAC key version {version} is not configured")
            }
            Self::InvalidVerifierLength(length) => {
                write!(formatter, "HMAC verifier must be 32 bytes, got {length}")
            }
        }
    }
}

impl std::error::Error for ApplicationSecretError {}
