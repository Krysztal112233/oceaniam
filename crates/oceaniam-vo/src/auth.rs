use garde::Validate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema, ts_rs::TS)]
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
    pub application_id: Uuid,

    pub auth: AuthVO,
}

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

pub type SystemSigninRequest = AuthVO;
pub type SystemSigninResponse = SigninResponse;
