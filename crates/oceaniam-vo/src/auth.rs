use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Validate, ToSchema, ts_rs::TS)]
#[serde(untagged)]
pub enum SignupRequest {
    /// Login via email
    Email {
        #[garde(email)]
        email: String,
        #[garde(skip)]
        pwd: String,
    },

    /// Login via phonenumber
    Phone {
        #[garde(phone_number)]
        phone: String,

        #[garde(skip)]
        pwd: String,
    },
}

#[derive(Debug, Deserialize, ToSchema, ts_rs::TS)]
pub struct SignupResponse {
    pub jwt: String,
}

pub type SigninRequest = SignupRequest;

pub type SigninResponse = SignupResponse;

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
