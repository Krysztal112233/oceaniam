use crate::error::Error;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, IntoActiveModel};
use uuid::Uuid;

use crate::{
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
            }
            .into_active_model()
            .insert(database)
            .await?
        };

        Ok(result)
    }
}

impl SubjectsHelper for Subjects {}
