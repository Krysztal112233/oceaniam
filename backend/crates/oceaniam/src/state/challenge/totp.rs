use oceaniam_common::error::Error;

use crate::state::challenge::{MfaValidator, ValidationContext};

#[derive(Debug, Clone)]
pub(crate) struct TotpValidator;

#[async_trait::async_trait]
impl MfaValidator for TotpValidator {
    async fn validate(&self, _ctx: ValidationContext) -> Result<(), Error> {
        todo!()
    }
}
