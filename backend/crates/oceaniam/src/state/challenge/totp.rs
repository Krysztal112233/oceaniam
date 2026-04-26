use std::sync::Arc;

use linkme::distributed_slice;
use oceaniam_challenge::validator::MfaValidator;
use oceaniam_common::error::Error;
use oceaniam_database::model::sea_orm_active_enums::ChallengeFactorType;
use sea_orm::DatabaseConnection;

use crate::state::challenge::{
    ConstructedMfaValidator, REGISTRY, ValidationContext, ValidatorFactorContext,
};

#[distributed_slice(REGISTRY)]
static TOTP_VALIDATOR: fn(ValidatorFactorContext) -> Result<ConstructedMfaValidator, Error> = init;

fn init(
    ValidatorFactorContext { database }: ValidatorFactorContext,
) -> Result<ConstructedMfaValidator, Error> {
    ConstructedMfaValidator {
        factor: ChallengeFactorType::Totp,
        validator: Arc::new(TotpValidator { database }),
    };
    todo!()
}

#[derive(Debug, Clone)]
struct TotpValidator {
    database: DatabaseConnection,
}

#[async_trait::async_trait]
impl MfaValidator for TotpValidator {
    type ValidationContext = ValidationContext;

    async fn validate(
        &self,
        ctx: Self::ValidationContext,
    ) -> Result<(), oceaniam_challenge::error::Error> {
        todo!()
    }
}
