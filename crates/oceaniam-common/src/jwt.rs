use jsonwebtoken::{Header, TokenData, Validation};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::Error;

pub trait JwtCodec<T>
where
    T: DeserializeOwned + Serialize,
{
    fn encode(&self, header: Header, claim: T) -> Result<String, Error>;
    fn decode(&self, jwt: &[u8], validation: &Validation) -> Result<TokenData<T>, Error>;
}

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

    /// JWT ID
    ///
    /// Unique identifier for the token, used to prevent replay attacks
    pub jti: Uuid,
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
    pub iat: i64,

    /// Issuer
    ///
    /// Token issuer (optional), e.g., "oceaniam-auth"
    pub iss: Option<String>,

    /// Audience
    ///
    /// Token audience (optional), represents the intended recipient of the token
    pub aud: Option<String>,

    /// JWT ID
    ///
    /// Unique identifier for the token, used to prevent replay attacks
    pub jti: Uuid,
}

pub trait ClaimHelper: DeserializeOwned + Serialize + Clone {
    fn new(sub: Uuid, ttl_seconds: i64, iss: Option<String>, aud: Option<String>) -> Self;

    fn decode(
        codec: Box<dyn JwtCodec<Self>>,
        jwt: impl Into<String>,
        validation: &Validation,
    ) -> Result<TokenData<Self>, Error> {
        codec.decode(jwt.into().as_bytes(), validation)
    }

    fn encode(self, header: Header, codec: Box<dyn JwtCodec<Self>>) -> Result<String, Error> {
        codec.encode(header, self)
    }
}

impl ClaimHelper for Claim {
    fn new(sub: Uuid, ttl_seconds: i64, iss: Option<String>, aud: Option<String>) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            sub,
            exp: now + ttl_seconds,
            iat: now,
            iss,
            aud,
            jti: Uuid::now_v7(),
        }
    }
}

impl ClaimHelper for SystemClaim {
    fn new(sub: Uuid, ttl_seconds: i64, iss: Option<String>, aud: Option<String>) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            sub,
            exp: now + ttl_seconds,
            iat: now,
            iss,
            aud,
            jti: Uuid::now_v7(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Jwk {
    pub kty: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub kid: Option<String>,

    #[serde(rename = "use", skip_serializing_if = "Option::is_none")]
    pub use_: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub alg: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub e: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct JwkSet {
    pub keys: Vec<Jwk>,
}

// TODO: OPTIMIZE here
impl From<JwkSet> for jsonwebtoken::jwk::JwkSet {
    fn from(value: JwkSet) -> Self {
        let value = serde_json::to_value(value).unwrap();

        serde_json::from_value(value).unwrap()
    }
}

// TODO: OPTIMIZE here
impl From<Jwk> for jsonwebtoken::jwk::Jwk {
    fn from(value: Jwk) -> Self {
        let value = serde_json::to_value(value).unwrap();

        serde_json::from_value(value).unwrap()
    }
}

// TODO: OPTIMIZE here
impl From<jsonwebtoken::jwk::Jwk> for Jwk {
    fn from(value: jsonwebtoken::jwk::Jwk) -> Self {
        let value = serde_json::to_value(value).unwrap();

        serde_json::from_value(value).unwrap()
    }
}
