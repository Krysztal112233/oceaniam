use axum::http::StatusCode;
use serde::Deserialize;

use crate::{
    error::Error,
    state::challenge::{MfaValidator, ValidationContext},
};

#[derive(Debug, Clone)]
pub(crate) struct EmailTotpValidator;

#[derive(Debug, Deserialize)]
struct EmailTotpPayload {
    code: String,
}

#[async_trait::async_trait]
impl MfaValidator for EmailTotpValidator {
    async fn validate(&self, ctx: ValidationContext) -> Result<(), Error> {
        let input: EmailTotpPayload = serde_json::from_value(ctx.input)
            .map_err(|e| Error::with_code(StatusCode::BAD_REQUEST, e.to_string()))?;

        let payload: EmailTotpPayload = serde_json::from_value(ctx.payload.ok_or_else(|| {
            Error::with_code(
                StatusCode::BAD_REQUEST,
                "email TOTP challenge payload is missing",
            )
        })?)
        .map_err(|e| Error::with_code(StatusCode::BAD_REQUEST, e.to_string()))?;

        if input.code == payload.code {
            Ok(())
        } else {
            Err(Error::with_code(
                StatusCode::UNAUTHORIZED,
                "invalid email TOTP code",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::EmailTotpValidator;
    use crate::error::Error;
    use crate::state::challenge::{MfaValidator, ValidationContext};

    // NOTE: AI-generated test
    #[tokio::test]
    async fn validate_accepts_matching_email_totp_code() {
        let validator = EmailTotpValidator;
        let ctx = ValidationContext {
            input: json!({ "code": "123456" }),
            payload: Some(json!({ "code": "123456" })),
            subject_id: Uuid::now_v7(),
        };

        validator
            .validate(ctx)
            .await
            .expect("matching email TOTP code should be accepted");
    }

    // NOTE: AI-generated test
    #[tokio::test]
    async fn validate_rejects_mismatched_email_totp_code() {
        let validator = EmailTotpValidator;
        let ctx = ValidationContext {
            input: json!({ "code": "654321" }),
            payload: Some(json!({ "code": "123456" })),
            subject_id: Uuid::now_v7(),
        };

        let err = validator
            .validate(ctx)
            .await
            .expect_err("mismatched email TOTP code should be rejected");

        assert!(matches!(err, Error::CustomMessage { code: 401, .. }));
    }
}
