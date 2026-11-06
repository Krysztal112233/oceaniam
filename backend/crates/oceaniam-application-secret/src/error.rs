use snafu::Snafu;

#[derive(Debug, Clone, PartialEq, Eq, Snafu)]
pub enum ApplicationSecretError {
    #[snafu(display(
        "application secret must be `app_` followed by 32 ASCII alphanumeric characters"
    ))]
    InvalidSecretFormat,
    #[snafu(display("HMAC key must be hexadecimal"))]
    InvalidKeyEncoding,
    #[snafu(display("HMAC key must contain exactly 32 bytes"))]
    InvalidKeyLength,
    #[snafu(display("HMAC key must not be all zeroes"))]
    AllZeroKey,
    #[snafu(display("invalid HMAC key entry; expected `version:hex` in the keys list"))]
    InvalidKeyEntry,
    #[snafu(display("invalid HMAC key version `{version}`"))]
    InvalidKeyVersionSyntax { version: String },
    #[snafu(display("HMAC key version must be positive, got {version}"))]
    InvalidKeyVersion { version: i32 },
    #[snafu(display("duplicate HMAC key version {version}"))]
    DuplicateKeyVersion { version: i32 },
    #[snafu(display("current HMAC key version {version} is not configured"))]
    MissingCurrentKey { version: i32 },
    #[snafu(display("HMAC key version {version} is not configured"))]
    MissingKeyVersion { version: i32 },
    #[snafu(display("HMAC verifier must be 32 bytes, got {length}"))]
    InvalidVerifierLength { length: usize },
}
