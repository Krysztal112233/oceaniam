use oceaniam_common::error::Error;
use sea_orm::{ActiveModelTrait, IntoActiveModel};
use uuid::Uuid;

use crate::{
    helper::SafeTransactionConnectionTrait,
    model::{self, prelude::Subjects, sea_orm_active_enums::SubjectTypeEnum},
};

#[async_trait::async_trait]
pub trait SubjectsHelper {
    async fn create_subjects(
        application_id: Uuid,
        typ: SubjectTypeEnum,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::subjects::Model, Error> {
        let result = {
            model::subjects::Model {
                ref_id: Uuid::now_v7(),
                r#type: typ,
                application_id,
            }
            .into_active_model()
            .insert(database)
            .await?
        };

        Ok(result)
    }
}

impl SubjectsHelper for Subjects {}
