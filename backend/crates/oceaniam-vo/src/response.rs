use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, Default, ToSchema)]
pub struct Empty {}

#[derive(Debug, Deserialize, Serialize, Default, ToSchema)]
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
