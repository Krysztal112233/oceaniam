use crate::error::Error;
use axum::http::StatusCode;
use chrono::{DateTime, Duration, FixedOffset, Utc};
use oceaniam_common::helpers::gen_random;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseBackend, EntityTrait, ExprTrait,
    IntoActiveModel, QueryFilter, QuerySelect, Statement, sea_query::Expr,
};
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
            expires_at: (Utc::now() + duration).into(),
            ..self
        }
    }
}

impl Default for CreateChallengeOpts {
    fn default() -> Self {
        Self {
            expires_at: (Utc::now() + Duration::seconds(30)).into(),
            max_attempts: Some(5),
            factor_type: ChallengeFactorType::Totp,
            challenge_purpose_type: ChallengePurposeType::Signin,
            payload: None,
        }
    }
}

#[async_trait::async_trait]
pub trait ChallengesHelper {
    #[tracing::instrument(
        level = "info",
        name = "db.challenges.get_challenge",
        skip_all,
        fields(otel.kind = "internal")
    )]
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

    #[tracing::instrument(
        level = "info",
        name = "db.challenges.create_challenge",
        skip_all,
        fields(otel.kind = "internal")
    )]
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

    #[tracing::instrument(
        level = "info",
        name = "db.challenges.consume_challenge",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn consume_challenge(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::challenges::Model, Error> {
        let challenge = Self::get_challenge(id, database).await?;

        Ok(model::challenges::ActiveModel {
            status: Set(ChallengeStatusType::Consumed),
            consumed_at: Set(Some(Utc::now().fixed_offset())),
            ..challenge.into_active_model()
        }
        .update(database)
        .await?)
    }

    /// Reads a still-usable pending challenge for re-rendering its form.
    ///
    /// The expiry predicate uses the database clock so it agrees with the OTP-step checks. A
    /// consumed, exhausted-agnostic but expired, or foreign-Application challenge is
    /// indistinguishable from an absent one.
    #[tracing::instrument(
        level = "info",
        name = "db.challenges.get_pending_challenge",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_pending_challenge(
        id: Uuid,
        application_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::challenges::Model, Error> {
        use model::challenges::Column::*;

        Challenges::find_by_id(id)
            .filter(ApplicationId.eq(application_id))
            .filter(Status.eq(ChallengeStatusType::Pending))
            .filter(Expr::col(ExpiresAt).gt(Expr::cust("clock_timestamp()")))
            .one(database)
            .await?
            .ok_or_else(|| challenge_not_found(id))
    }

    /// Locks a pending challenge `FOR UPDATE` for one OTP attempt inside the caller's
    /// transaction.
    ///
    /// The lock is taken only after the Application-binding, pending-status, and
    /// attempts-remaining predicates match; expiry is then re-sampled with `clock_timestamp()`
    /// so time spent waiting for the lock cannot revive an expired challenge.
    #[tracing::instrument(
        level = "info",
        name = "db.challenges.lock_challenge_for_attempt",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn lock_challenge_for_attempt(
        id: Uuid,
        application_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::challenges::Model, Error> {
        use model::challenges::Column::*;

        let challenge = Challenges::find_by_id(id)
            .filter(ApplicationId.eq(application_id))
            .filter(Status.eq(ChallengeStatusType::Pending))
            .filter(Expr::col(AttemptCount).lt(Expr::col(MaxAttempts)))
            .lock_exclusive()
            .one(database)
            .await?
            .ok_or_else(|| challenge_not_found(id))?;

        // PostgreSQL's CURRENT_TIMESTAMP is fixed at transaction start. Sample the wall clock
        // only after acquiring the row lock so time spent waiting cannot revive an expired
        // challenge.
        let now: DateTime<FixedOffset> = database
            .query_one_raw(Statement::from_string(
                DatabaseBackend::Postgres,
                "SELECT clock_timestamp() AS now".to_owned(),
            ))
            .await?
            .ok_or(Error::from(sea_orm::DbErr::RecordNotFound(
                "clock_timestamp".to_owned(),
            )))?
            .try_get("", "now")?;
        if challenge.expires_at <= now {
            return Err(challenge_not_found(id));
        }

        Ok(challenge)
    }

    /// Atomically counts one failed OTP attempt against a locked challenge.
    ///
    /// The conditional increment never counts past `max_attempts` and never revives a consumed
    /// challenge; the caller holds the row lock from
    /// [`Self::lock_challenge_for_attempt`].
    #[tracing::instrument(
        level = "info",
        name = "db.challenges.increment_challenge_attempt",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn increment_challenge_attempt(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::challenges::Model, Error> {
        use model::challenges::Column::*;

        let mut updated = Challenges::update_many()
            .col_expr(AttemptCount, Expr::col(AttemptCount).add(1))
            .filter(Id.eq(id))
            .filter(Status.eq(ChallengeStatusType::Pending))
            .filter(Expr::col(AttemptCount).lt(Expr::col(MaxAttempts)))
            .exec_with_returning(database)
            .await?;

        debug_assert!(
            updated.len() <= 1,
            "a primary-key-filtered update returned multiple challenges"
        );
        updated.pop().ok_or_else(|| challenge_not_found(id))
    }

    /// Consumes a locked challenge exactly once after a successful OTP verification.
    ///
    /// The conditional update re-checks the pending status and the database-clock expiry so the
    /// challenge cannot be consumed twice or after it expired; the caller holds the row lock
    /// from [`Self::lock_challenge_for_attempt`].
    #[tracing::instrument(
        level = "info",
        name = "db.challenges.consume_pending_challenge",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn consume_pending_challenge(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::challenges::Model, Error> {
        use model::challenges::Column::*;

        let mut updated = Challenges::update_many()
            .col_expr(Status, Expr::value(ChallengeStatusType::Consumed))
            .col_expr(ConsumedAt, Expr::cust("clock_timestamp()"))
            .filter(Id.eq(id))
            .filter(Status.eq(ChallengeStatusType::Pending))
            .filter(Expr::col(ExpiresAt).gt(Expr::cust("clock_timestamp()")))
            .exec_with_returning(database)
            .await?;

        debug_assert!(
            updated.len() <= 1,
            "a primary-key-filtered update returned multiple challenges"
        );
        updated.pop().ok_or_else(|| challenge_not_found(id))
    }
}

fn challenge_not_found(id: Uuid) -> Error {
    Error::with_code(
        StatusCode::NOT_FOUND,
        format!("challenge id={id} not found"),
    )
}

impl ChallengesHelper for Challenges {}

pub trait ChallengePayload: Serialize + DeserializeOwned {}

impl<T> ChallengePayload for T where T: Serialize + DeserializeOwned {}
