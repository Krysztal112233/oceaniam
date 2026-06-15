use crate::error::Error;
use axum::http::StatusCode;
use serde::Deserialize;

use crate::state::challenge::{MfaValidator, ValidationContext};
use crate::state::credentials::ManagedCredentialVaults;

#[derive(Debug, Clone)]
pub(crate) struct TotpValidator {
    pub credentials: ManagedCredentialVaults,
}

#[derive(Debug, Deserialize)]
struct ChallengePayload {
    code: String,
}

#[async_trait::async_trait]
impl MfaValidator for TotpValidator {
    async fn validate(&self, ctx: ValidationContext) -> Result<(), Error> {
        let payload: ChallengePayload = serde_json::from_value(ctx.input)
            .map_err(|e| Error::with_code(StatusCode::BAD_REQUEST, e.to_string()))?;

        let ok = self
            .credentials
            .verify_totp(ctx.subject_id, &payload.code)
            .await?;

        if ok {
            Ok(())
        } else {
            Err(Error::with_code(
                StatusCode::UNAUTHORIZED,
                "invalid TOTP code",
            ))
        }
    }
}
