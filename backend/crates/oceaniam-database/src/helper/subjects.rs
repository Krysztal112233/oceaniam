use axum::http::StatusCode;
use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::sea_query::Expr;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};
use uuid::Uuid;

use crate::{
    error::Error,
    helper::SafeTransactionConnectionTrait,
    helper::subject_roles::SubjectRolesHelper,
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
            .ok_or_else(|| {
                Error::with_code(StatusCode::NOT_FOUND, format!("subject {id} not found"))
            })
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
        let result = {
            model::subjects::Model {
                id,
                r#type: typ,
                application_id,
                created_at: Utc::now().into(),
                expires_at: None,
            }
            .into_active_model()
            .insert(database)
            .await?
        };

        Ok(result)
    }

    /// Sets the expiration timestamp on an existing subject.
    ///
    /// Used by the development-account flow: the subject row is created as a normal account
    /// first (reusing the regular creation ordering), then stamped with `expires_at` inside the
    /// same transaction.
    #[tracing::instrument(
        level = "info",
        name = "db.subjects.set_subject_expiration",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn set_subject_expiration(
        id: Uuid,
        expires_at: DateTime<FixedOffset>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<(), Error> {
        Subjects::update_many()
            .col_expr(model::subjects::Column::ExpiresAt, Expr::value(expires_at))
            .filter(model::subjects::Column::Id.eq(id))
            .exec(database)
            .await?;

        Ok(())
    }

    /// Lazy expiration rejection for time-limited (development) accounts.
    ///
    /// Returns 401 when the subject does not exist or its `expires_at` is in the past. Normal
    /// accounts (`expires_at IS NULL`) always pass. The message is deliberately the generic
    /// login-failure one so this check does not leak account existence (see
    /// `docs/design/ERROR_CODES.md`).
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
        let subject = Subjects::find_by_id(id)
            .one(database)
            .await?
            .ok_or_else(|| {
                Error::with_code(
                    StatusCode::UNAUTHORIZED,
                    oceaniam_common::consts::USER_LOGIN_FAILED_MSG,
                )
            })?;

        if let Some(expires_at) = subject.expires_at
            && expires_at <= Utc::now()
        {
            return Err(Error::with_code(
                StatusCode::UNAUTHORIZED,
                oceaniam_common::consts::USER_LOGIN_FAILED_MSG,
            ));
        }

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
            .ok_or_else(|| {
                Error::with_code(StatusCode::NOT_FOUND, format!("subject {id} not found"))
            })?;

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
