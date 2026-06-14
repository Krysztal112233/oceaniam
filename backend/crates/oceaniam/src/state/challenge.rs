use std::{collections::HashMap, fmt::Debug, sync::Arc, time::Duration};

use crate::error::Error;
use axum::http::StatusCode;
use chrono::Utc;
use moka::future::Cache;
use oceaniam_audit::types::{AuditPayload, CreateChallengePayload, VerifyChallengePayload};
use oceaniam_database::{
    helper::{
        SafeTransactionConnectionTrait,
        challenges::{ChallengesHelper, CreateChallengeOpts},
    },
    model::{
        challenges::Model as ChallengeModel,
        prelude::Challenges,
        sea_orm_active_enums::{ChallengeFactorType, ChallengeStatusType},
    },
};
use sea_orm::DatabaseConnection;
use serde_json::Value;
use tracing::{info, warn};
use uuid::Uuid;

use super::audit::Auditing;
use crate::state::credentials::ManagedCredentialVaults;

mod totp;

#[async_trait::async_trait]
pub trait MfaValidator: Sync + Send + Debug {
    async fn validate(&self, ctx: ValidationContext) -> Result<(), Error>;
}

#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub input: Value,
    pub subject_id: Uuid,
}

type SharedMfaValidator = Arc<dyn MfaValidator>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ChallengeRecord {
    application_id: Uuid,
    id: Uuid,
}

#[derive(Debug, Clone)]
pub struct ManagedChallenges {
    application_id: Uuid,

    auditing: Auditing,
    database: DatabaseConnection,
    validators: HashMap<ChallengeFactorType, SharedMfaValidator>,
    cache: Cache<ChallengeRecord, ChallengeModel>,
}

impl ManagedChallenges {
    pub fn new(
        application_id: Uuid,
        database: DatabaseConnection,
        auditing: Auditing,
        credentials: ManagedCredentialVaults,
    ) -> Self {
        let mut validators: HashMap<ChallengeFactorType, SharedMfaValidator> = HashMap::new();
        validators.insert(
            ChallengeFactorType::Totp,
            Arc::new(totp::TotpValidator { credentials }),
        );

        Self {
            application_id,
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
        subject_id: Uuid,
        opts: CreateChallengeOpts,
    ) -> Result<ChallengeModel, Error> {
        let challenge =
            Challenges::create_challenge(self.application_id, subject_id, opts, &self.database)
                .await?;

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
                    application_id: self.application_id,
                    id: challenge.id,
                },
                challenge.clone(),
            )
            .await;

        Ok(challenge)
    }

    pub async fn get_challenge(&self, id: Uuid) -> Result<ChallengeModel, Error> {
        let record = ChallengeRecord {
            application_id: self.application_id,
            id,
        };

        let challenge = self
            .cache
            .try_get_with(record, async {
                let challenge = Challenges::get_challenge(id, &self.database).await?;

                if challenge.application_id != self.application_id {
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

    pub async fn set_pass(&self, id: Uuid) -> Result<(), Error> {
        self.set_pass_in_tx(id, &self.database).await
    }

    pub async fn set_pass_in_tx(
        &self,
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        let record = ChallengeRecord {
            application_id: self.application_id,
            id,
        };

        self.cache
            .try_get_with(record.clone(), async {
                let challenge = Challenges::get_challenge(id, &self.database).await?;

                // NOTE: FOR SECURITY CONSIDERATION
                if challenge.application_id != self.application_id {
                    return Err(Error::with_code(
                        StatusCode::NOT_FOUND,
                        format!("challenge id={id} not found"),
                    ));
                }

                Ok(challenge)
            })
            .await?;

        let updated_challenge = Challenges::consume_challenge(id, database).await?;

        self.cache.insert(record, updated_challenge).await;

        Ok(())
    }

    #[tracing::instrument(skip(self, payload), fields(challenge_id = %id))]
    pub async fn verify_challenge(&self, id: Uuid, payload: Value) -> Result<(), Error> {
        let challenge = self.get_challenge(id).await?;
        let factor_type = challenge.factor_type.clone();

        let validator = self.validators.get(&factor_type).ok_or_else(|| {
            warn!(
                %factor_type,
                application_id = %challenge.application_id,
                subject_id = %challenge.subject_id,
                "no validator configured for factor type"
            );

            Error::with_code(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("validator for factor_type={factor_type} is not configured"),
            )
        })?;

        validator
            .validate(ValidationContext {
                subject_id: challenge.subject_id,
                input: payload,
            })
            .await
            .map_err(|e| {
                warn!(
                    %factor_type,
                    application_id = %challenge.application_id,
                    subject_id = %challenge.subject_id,
                    error = %e,
                    "challenge verification failed"
                );

                Error::with_code(
                    StatusCode::UNAUTHORIZED,
                    format!("challenge verification failed: {e}"),
                )
            })?;

        self.set_pass(id).await?;

        info!(
            challenge_id = %challenge.id,
            application_id = %challenge.application_id,
            subject_id = %challenge.subject_id,
            %factor_type,
            purpose = %challenge.purpose,
            "challenge verified successfully"
        );

        self.auditing
            .write(AuditPayload::from(VerifyChallengePayload {
                challenge_id: challenge.id,
                application_id: challenge.application_id,
                subject_id: challenge.subject_id,
                factor_type: challenge.factor_type,
                purpose: challenge.purpose,
            }))
            .await;

        Ok(())
    }
}
