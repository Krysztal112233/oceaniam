use std::{sync::Arc, time::Duration};

use axum::http::StatusCode;
use chrono::Utc;
use linkme::distributed_slice;
use moka::future::Cache;
use oceaniam_audit::types::{AuditPayload, CreateChallengePayload};
use oceaniam_challenge::validator::{MfaValidator, ValidatorRegistry};
use oceaniam_common::error::Error;
use oceaniam_database::{
    helper::{
        SafeTransactionConnectionTrait,
        challenges::{ChallengesHelper, CreateChallengeOpts},
    },
    model::{
        challenges::{ActiveModel as ChallengeActiveModel, Model as ChallengeModel},
        prelude::Challenges,
        sea_orm_active_enums::{ChallengeFactorType, ChallengeStatusType},
    },
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, IntoActiveModel};
use uuid::Uuid;

use super::audit::Auditing;

#[allow(unused)]
mod totp;

#[derive(Debug, Clone)]
pub struct ValidatorFactorContext {
    database: DatabaseConnection,
}

#[derive(Debug, Clone)]
pub struct ValidationContext {}

type SharedMfaValidator = Arc<dyn MfaValidator<ValidationContext = ValidationContext>>;
type ChallengeValidatorRegistry = ValidatorRegistry<ValidationContext>;
type ValidatorFactory =
    fn(context: ValidatorFactorContext) -> Result<ConstructedMfaValidator, Error>;

pub(crate) struct ConstructedMfaValidator {
    pub factor: ChallengeFactorType,
    pub validator: SharedMfaValidator,
}

#[distributed_slice]
static REGISTRY: [ValidatorFactory];

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ChallengeRecord {
    application_id: Uuid,
    id: Uuid,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct ManagedChallenges {
    auditing: Auditing,
    database: DatabaseConnection,
    validators: ChallengeValidatorRegistry,
    cache: Cache<ChallengeRecord, ChallengeModel>,
}

impl ManagedChallenges {
    pub fn new(database: DatabaseConnection, auditing: Auditing) -> Self {
        let ctx = ValidatorFactorContext {
            database: database.clone(),
        };

        let validators = REGISTRY
            .iter()
            .map(|it| it(ctx.clone()))
            .filter(|it| it.is_ok())
            .map(Result::unwrap)
            .map(|ConstructedMfaValidator { factor, validator }| (factor, validator))
            .collect();

        let validators = ChallengeValidatorRegistry::new(validators);

        Self {
            auditing,
            database,
            cache: Cache::builder()
                .time_to_idle(Duration::from_mins(5))
                .build(),
            validators,
        }
    }

    pub async fn create_challenge(
        &self,
        application_id: Uuid,
        subject_id: Uuid,
        opts: CreateChallengeOpts,
    ) -> Result<ChallengeModel, Error> {
        let challenge =
            Challenges::create_challenge(application_id, subject_id, opts, &self.database).await?;

        self.auditing
            .write(AuditPayload::from(CreateChallengePayload {
                challenge_id: challenge.id,
                application_id: challenge.application_id,
                subject_id: challenge.subject_id,
                factor_type: challenge.factor_type.clone(),
                purpose: challenge.purpose.clone(),
            }))
            .await;

        self.cache
            .insert(
                ChallengeRecord {
                    application_id,
                    id: challenge.id,
                },
                challenge.clone(),
            )
            .await;

        Ok(challenge)
    }

    pub async fn get_challenge(
        &self,
        application_id: Uuid,
        id: Uuid,
    ) -> Result<ChallengeModel, Error> {
        let record = ChallengeRecord { application_id, id };

        let challenge = self
            .cache
            .try_get_with(record, async {
                let challenge = Challenges::get_challenge(id, &self.database).await?;

                if challenge.application_id != application_id {
                    return Err(Error::with_code(
                        StatusCode::NOT_FOUND,
                        format!("challenge id={id} not found"),
                    ));
                }

                Ok(challenge)
            })
            .await?;

        if challenge.status != ChallengeStatusType::Pending
            || challenge.consumed_at.is_some()
            || challenge.expires_at <= Utc::now().fixed_offset()
        {
            return Err(Error::with_code(
                StatusCode::NOT_FOUND,
                format!("challenge id={id} not found"),
            ));
        }

        Ok(challenge)
    }

    pub async fn set_pass(&self, application_id: Uuid, id: Uuid) -> Result<(), Error> {
        self.set_pass_in_tx(application_id, id, &self.database)
            .await
    }

    pub async fn set_pass_in_tx(
        &self,
        application_id: Uuid,
        id: Uuid,

        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        let record = ChallengeRecord { application_id, id };

        let challenge = self
            .cache
            .try_get_with(record.clone(), async {
                let challenge = Challenges::get_challenge(id, &self.database).await?;

                // NOTE: FOR SECURITY CONSIDERATION
                if challenge.application_id != application_id {
                    return Err(Error::with_code(
                        StatusCode::NOT_FOUND,
                        format!("challenge id={id} not found"),
                    ));
                }

                Ok(challenge)
            })
            .await?;

        let challenge = ChallengeActiveModel {
            status: Set(ChallengeStatusType::Consumed),
            consumed_at: Set(Some(Utc::now().fixed_offset())),
            ..challenge.into_active_model()
        };

        let updated_challenge = challenge.update(database).await?;

        self.cache.insert(record, updated_challenge).await;

        Ok(())
    }
}
