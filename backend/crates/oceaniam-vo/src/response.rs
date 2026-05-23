use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, Default, ToSchema, TS)]
pub struct Empty {}

#[derive(Debug, Deserialize, Serialize, Default, ToSchema, TS)]
pub struct ErrorResponse {
    msg: String,
    error_id: String,
}

impl ErrorResponse {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            msg: msg.into(),
            error_id: String::new(),
        }
    }
}
