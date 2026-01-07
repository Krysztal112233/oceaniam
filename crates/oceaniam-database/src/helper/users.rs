use oceaniam_common::error::Error;
use oceaniam_credential::CredentialVault;
use sea_orm::{ActiveModelTrait, IntoActiveModel};
use uuid::Uuid;

use crate::{
    helper::{
        SafeTransactionConnectionTrait, credentials::CredentialsHelper, subjects::SubjectsHelper,
    },
    model::{
        self,
        prelude::{Credentials, Subjects},
        sea_orm_active_enums::SubjectTypeEnum,
    },
};

#[async_trait::async_trait]
pub trait UserHelper {
    async fn create_user(
        name: impl Into<String> + Send,
        credential: CredentialVault,
        application_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::users::Model, Error> {
        let database = database.begin().await?;
        let result = {
            let subject =
                Subjects::create_subjects(application_id, SubjectTypeEnum::User, &database).await?;
            let _ = Credentials::create_cradentials(subject.ref_id, credential, &database).await?;

            model::users::Model {
                id: Uuid::now_v7(),
                name: name.into(),
                application_id,
            }
            .into_active_model()
            .insert(&database)
            .await?
        };
        database.commit().await?;

        Ok(result)
    }
}
