use std::time::Duration;

use axum::http::StatusCode;
use chrono::Utc;
use moka::future::Cache;
use oceaniam_common::error::Error;
use oceaniam_database::{
    helper::{
        SafeTransactionConnectionTrait,
        challenges::{ChallengesHelper, CreateChallengeOpts},
    },
    model::{self, prelude::Challenges, sea_orm_active_enums::ChallengeStatusType},
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, IntoActiveModel};
use uuid::Uuid;

use model::challenges::ActiveModel as ChallengeActiveModel;
use model::challenges::Model as ChallengeModel;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ChallengeRecord {
    application_id: Uuid,
    id: Uuid,
}

#[derive(Debug, Clone)]
pub struct ManagedChallenges {
    database: DatabaseConnection,
    cache: Cache<ChallengeRecord, ChallengeModel>,
}

impl ManagedChallenges {
    pub fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            cache: Cache::builder()
                .time_to_idle(Duration::from_mins(5))
                .build(),
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

        let challenge = challenge.update(database).await?;

        self.cache.insert(record, challenge).await;

        Ok(())
    }
}
