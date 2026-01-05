use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema, ts_rs::TS)]
#[serde(untagged)]
pub enum SignupRequest {
    /// Login via email
    Email { email: String, pwd: String },

    /// Login via phone
    Phone { phone: String, pwd: String },
}

#[derive(Debug, Deserialize, ToSchema, ts_rs::TS)]
pub struct SignupResponse {}

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
