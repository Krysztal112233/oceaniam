use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Claim - Used for issuing JWT tokens to external applications/clients
///
/// This claim structure is used when IAM (Identity and Access Management) issues
/// JWT tokens to external applications or clients that need to authenticate
/// and access resources through the IAM system.
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, ts_rs::TS)]
pub struct Claim {
    /// Subject
    ///
    /// The subject of the token, typically the user's unique identifier (e.g., UUID)
    pub sub: Uuid,

    /// Expiration Time
    ///
    /// Token expiration time (Unix timestamp, seconds)
    pub exp: i64,

    /// Issued At
    ///
    /// Token issuance time (Unix timestamp, seconds)
    pub iat: i64,

    /// Issuer
    ///
    /// Token issuer (optional), e.g., "oceaniam-auth"
    pub iss: Option<String>,

    /// Audience
    ///
    /// Token audience (optional), represents the intended recipient of the token
    pub aud: Option<String>,
}

/// SystemClaim - Used for IAM's internal authentication
///
/// This claim structure is used for internal authentication within the IAM system itself,
/// such as for inter-service communication, internal system operations, and
/// administrative tasks that require elevated privileges.
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, ts_rs::TS)]
pub struct SystemClaim {
    /// Subject
    ///
    /// The subject of the token, typically the user's unique identifier (e.g., UUID)
    pub sub: Uuid,

    /// Expiration Time
    ///
    /// Token expiration time (Unix timestamp, seconds)
    pub exp: i64,

    /// Issued At
    ///
    /// Token issuance time (Unix timestamp, seconds)
    #[serde(rename = "iat")]
    pub iat: i64,

    /// Issuer
    ///
    /// Token issuer (optional), e.g., "oceaniam-auth"
    pub iss: Option<String>,

    /// Audience
    ///
    /// Token audience (optional), represents the intended recipient of the token
    pub aud: Option<String>,
}
