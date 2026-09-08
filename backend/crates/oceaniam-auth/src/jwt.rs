use serde::{Deserialize, Serialize, de::DeserializeOwned};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{Header, TokenData, Validation, error::Error};

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
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Claim {
    /// Subject
    ///
    /// The user's stable external subject (`users.oidc_sub`), serialized as a UUID.
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
    pub aud: Option<Vec<String>>,

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
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
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
    pub aud: Option<Vec<String>>,

    /// JWT ID
    ///
    /// Unique identifier for the token, used to prevent replay attacks
    pub jti: Uuid,
}

pub trait ClaimHelper: DeserializeOwned + Serialize + Clone {
    fn new(sub: Uuid, ttl_seconds: i64, iss: Option<String>, aud: Option<Vec<String>>) -> Self;

    fn jti(&self) -> Uuid;

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
    fn new(sub: Uuid, ttl_seconds: i64, iss: Option<String>, aud: Option<Vec<String>>) -> Self {
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

    fn jti(&self) -> Uuid {
        self.jti
    }
}

impl ClaimHelper for SystemClaim {
    fn new(sub: Uuid, ttl_seconds: i64, iss: Option<String>, aud: Option<Vec<String>>) -> Self {
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

    fn jti(&self) -> Uuid {
        self.jti
    }
}

#[derive(Debug, Clone)]
pub struct JwtValidator(Validation);

impl JwtValidator {
    pub fn new(validation: Validation) -> Self {
        Self(validation)
    }
}

impl std::ops::Deref for JwtValidator {
    type Target = Validation;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{Claim, ClaimHelper, SystemClaim};
    use uuid::Uuid;

    // NOTE: AI-generated test
    #[test]
    fn claim_construction_preserves_legacy_wire_fields_and_five_day_ttl() {
        let subject =
            Uuid::parse_str("018f47a6-7b53-7cc0-8f5c-5c8bf2d6b8cf").expect("fixed subject UUID");
        let issuer = Some("https://issuer.oceaniam.test".to_owned());
        let audience = Some(vec!["client-a".to_owned(), "client-b".to_owned()]);
        let claim = Claim::new(subject, 5 * 24 * 60 * 60, issuer.clone(), audience.clone());
        let system_claim =
            SystemClaim::new(subject, 5 * 24 * 60 * 60, issuer.clone(), audience.clone());

        assert_eq!(claim.exp - claim.iat, 432_000);
        assert_eq!(system_claim.exp - system_claim.iat, 432_000);
        assert_eq!(claim.sub, subject);
        assert_eq!(system_claim.sub, subject);
        assert_eq!(claim.iss, issuer);
        assert_eq!(claim.aud, audience);
        assert_eq!(claim.jti.get_version_num(), 7);
        assert_eq!(system_claim.jti.get_version_num(), 7);

        let value = serde_json::to_value(&claim).expect("serialize claim");
        assert_eq!(value["sub"], json!(subject));
        assert!(value["exp"].is_i64());
        assert!(value["iat"].is_i64());
        assert_eq!(value["iss"], json!("https://issuer.oceaniam.test"));
        assert_eq!(value["aud"], json!(["client-a", "client-b"]));
        assert!(value["jti"].is_string());

        let without_policy = SystemClaim::new(subject, 60, None, None);
        let value = serde_json::to_value(without_policy).expect("serialize system claim");
        assert!(value["iss"].is_null());
        assert!(value["aud"].is_null());
    }
}
