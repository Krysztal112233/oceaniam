use axum::http::StatusCode;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};
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
            }
            .into_active_model()
            .insert(database)
            .await?
        };

        Ok(result)
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
