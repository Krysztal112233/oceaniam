use axum::http::StatusCode;
use chrono::Duration;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, ExprTrait, IntoActiveModel, QueryFilter,
};
use uuid::Uuid;

use crate::{
    error::Error,
    helper::subject_roles::SubjectRolesHelper,
    helper::{SafeTransactionConnectionTrait, db_clock_now, exactly_one_returned},
    model::{
        self,
        prelude::{SubjectRoles, Subjects},
        sea_orm_active_enums::SubjectTypeEnum,
    },
};

#[async_trait::async_trait]
pub trait SubjectsHelper {
    #[tracing::instrument(
        level = "info",
        name = "db.subjects.get_subject_by_id",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn get_subject_by_id(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::subjects::Model, Error> {
        Subjects::find_by_id(id)
            .one(database)
            .await?
            .ok_or_else(|| Error::not_found(format!("subject {id} not found")))
    }

    #[tracing::instrument(
        level = "info",
        name = "db.subjects.create_subjects",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn create_subjects(
        id: Uuid,
        application_id: Uuid,
        typ: SubjectTypeEnum,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::subjects::Model, Error> {
        let created_at = db_clock_now(database).await?;
        let result = {
            model::subjects::Model {
                id,
                r#type: typ,
                application_id,
                created_at,
                expires_at: None,
            }
            .into_active_model()
            .insert(database)
            .await?
        };

        Ok(result)
    }

    /// Sets the expiration timestamp on an existing subject from a time-to-live.
    ///
    /// Used by the development-account flow: the subject row is created as a normal account
    /// first (reusing the regular creation ordering), then stamped with `expires_at` inside the
    /// same transaction. The expiry is computed on the database clock (Rule 3):
    /// `expires_at = clock_timestamp() + ttl`, so a negative or zero ttl stamps an
    /// immediately-expired subject — a legal state. The primary-key-filtered write returns the
    /// updated row and reports a not-found error when the subject is gone instead of silently
    /// no-oping.
    #[tracing::instrument(
        level = "info",
        name = "db.subjects.set_subject_expiration",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn set_subject_expiration(
        id: Uuid,
        ttl: Duration,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::subjects::Model, Error> {
        // The ttl is bound as whole microseconds (timestamptz resolution); only absurd
        // durations beyond ~292k years saturate.
        let microseconds = ttl.num_microseconds().unwrap_or(i64::MAX);
        let updated = Subjects::update_many()
            .filter(model::subjects::Column::Id.eq(id))
            .col_expr(
                model::subjects::Column::ExpiresAt,
                Expr::cust_with_values(
                    "clock_timestamp() + $1::bigint * interval '1 microsecond'",
                    [microseconds],
                ),
            )
            .exec_with_returning(database)
            .await?;

        exactly_one_returned(updated, || {
            Error::not_found(format!("subject {id} not found"))
        })
    }

    /// Lazy expiration rejection for time-limited (development) accounts.
    ///
    /// Returns 401 when the subject does not exist or its `expires_at` is in the past. Normal
    /// accounts (`expires_at IS NULL`) always pass. The expiry predicate is pushed into the
    /// `WHERE` clause and evaluated on the database clock (Rule 3). The message is
    /// deliberately the generic login-failure one so this check does not leak account
    /// existence (see `docs/design/ERROR_CODES.md`).
    #[tracing::instrument(
        level = "info",
        name = "db.subjects.ensure_subject_active",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn ensure_subject_active(
        id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        use model::subjects::Column::*;

        Subjects::find_by_id(id)
            .filter(
                Expr::col(ExpiresAt)
                    .is_null()
                    .or(Expr::col(ExpiresAt).gt(Expr::cust("clock_timestamp()"))),
            )
            .one(database)
            .await?
            .ok_or_else(|| {
                Error::with_code(
                    StatusCode::UNAUTHORIZED,
                    oceaniam_common::consts::USER_LOGIN_FAILED_MSG,
                )
            })?;

        Ok(())
    }

    #[tracing::instrument(
        level = "info",
        name = "db.subjects.resolve_subject_roles",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn resolve_subject_roles(
        id: Uuid,
        application_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Vec<Uuid>, Error> {
        let subject = Subjects::find_by_id(id)
            .one(database)
            .await?
            .ok_or_else(|| Error::not_found(format!("subject {id} not found")))?;

        if subject.application_id != application_id {
            return Err(Error::with_code(
                StatusCode::FORBIDDEN,
                format!("subject {id} does not belong to application {application_id}"),
            ));
        }

        SubjectRoles::get_subject_role_ids(id, database).await
    }
}

impl SubjectsHelper for Subjects {}
