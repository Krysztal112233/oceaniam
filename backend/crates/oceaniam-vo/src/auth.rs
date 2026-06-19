use chrono::{DateTime, FixedOffset};
use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema, PartialEq, Eq, ts_rs::TS)]
#[serde(untagged)]
pub enum AuthVO {
    /// Login via email
    Email {
        #[garde(email)]
        email: String,

        #[garde(skip)]
        password: String,
    },

    /// Login via phonenumber
    Phone {
        #[garde(phone_number)]
        phone: String,

        #[garde(skip)]
        password: String,
    },
}

#[derive(Debug, Serialize, Deserialize, ToSchema, ts_rs::TS)]
pub struct SignupResponse {
    pub jwt: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, ts_rs::TS)]
pub struct SigninRequest {
    pub auth: AuthVO,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, ts_rs::TS)]
pub struct SigninChallenge {
    pub challenge_id: String,
    pub factor_type: String,
    pub expires_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, ts_rs::TS)]
pub struct EnrollTotpResponse {
    pub provisioning_uri: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, ts_rs::TS)]
pub struct VerifyTotpRequest {
    pub code: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, ts_rs::TS)]
#[serde(untagged)]
pub enum SigninResponseOrChallenge {
    Signup(SignupResponse),
    Challenge(SigninChallenge),
}

#[derive(Debug, Serialize, ToSchema, ts_rs::TS)]
pub struct SignoutResponse {
    msg: String,
}

impl Default for SignoutResponse {
    fn default() -> Self {
        Self {
            msg: "farewell :)".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, ts_rs::TS, Validate)]
#[serde(untagged)]
pub enum SystemSigninRequest {
    /// Login via username
    Name {
        #[garde(skip)]
        name: String,

        #[garde(skip)]
        password: String,
    },
}
pub type SystemSigninResponseSchema = SigninResponseOrChallenge;
