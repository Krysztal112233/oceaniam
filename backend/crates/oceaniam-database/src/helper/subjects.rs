use axum::http::StatusCode;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};
use uuid::Uuid;

use crate::{
    error::Error,
    helper::SafeTransactionConnectionTrait,
    model::{self, prelude::Subjects, sea_orm_active_enums::SubjectTypeEnum},
};

#[async_trait::async_trait]
pub trait SubjectsHelper {
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
                application_role_id: None,
            }
            .into_active_model()
            .insert(database)
            .await?
        };

        Ok(result)
    }

    async fn resolve_subject_role(
        id: Uuid,
        application_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<Option<Uuid>, Error> {
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

        Ok(subject.application_role_id)
    }
}

impl SubjectsHelper for Subjects {}
