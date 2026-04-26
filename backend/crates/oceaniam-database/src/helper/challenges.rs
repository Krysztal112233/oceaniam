use std::ops::Add;

use axum::http::StatusCode;
use chrono::{Duration, Utc};
use oceaniam_common::{error::Error, helpers::gen_random};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    helper::SafeTransactionConnectionTrait,
    model::{self, prelude::Challenges, sea_orm_active_enums::*},
};

#[derive(Debug)]
pub struct CreateChallengeOpts {
    pub expires_at: sea_orm::prelude::DateTimeWithTimeZone,
    pub max_attempts: Option<i32>,
    pub factor_type: ChallengeFactorType,
    pub challenge_purpose_type: ChallengePurposeType,
    pub payload: Option<Value>,
}

impl CreateChallengeOpts {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn payload(self, payload: impl ChallengePayload) -> Result<Self, serde_json::Error> {
        Ok(Self {
            payload: Some(serde_json::to_value(payload)?),
            ..self
        })
    }

    pub fn expires_after(self, duration: Duration) -> Self {
        Self {
            expires_at: Utc::now().add(duration).into(),
            ..self
        }
    }
}

impl Default for CreateChallengeOpts {
    fn default() -> Self {
        Self {
            expires_at: Utc::now().add(Duration::seconds(30)).into(),
            max_attempts: Some(5),
            factor_type: ChallengeFactorType::Totp,
            challenge_purpose_type: ChallengePurposeType::Signin,
            payload: None,
        }
    }
}

#[async_trait::async_trait]
pub trait ChallengesHelper {
    async fn get_challenge(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::challenges::Model, Error> {
        Challenges::find_by_id(id)
            .one(database)
            .await?
            .ok_or_else(|| {
                Error::with_code(
                    StatusCode::NOT_FOUND,
                    format!("challenge id={id} not found"),
                )
            })
    }

    async fn create_challenge(
        application_id: Uuid,
        subject_id: Uuid,

        CreateChallengeOpts {
            expires_at,
            max_attempts,
            factor_type,
            challenge_purpose_type,
            payload,
        }: CreateChallengeOpts,
        transaction: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::challenges::Model, Error> {
        let token = gen_random(16);

        Ok(model::challenges::ActiveModel {
            id: Set(Uuid::now_v7()),
            application_id: Set(application_id),
            subject_id: Set(subject_id),
            token: Set(token),
            factor_type: Set(factor_type),
            purpose: Set(challenge_purpose_type),
            status: Set(ChallengeStatusType::Pending),
            attempt_count: Set(0),
            max_attempts: Set(max_attempts.unwrap_or(5)),
            expires_at: Set(expires_at),
            consumed_at: Set(None),
            created_at: Set(Utc::now().into()),
            payload: Set(payload),
        }
        .insert(transaction)
        .await?)
    }
}

impl ChallengesHelper for Challenges {}

pub trait ChallengePayload: Serialize + DeserializeOwned {}

impl<T> ChallengePayload for T where T: Serialize + DeserializeOwned {}
