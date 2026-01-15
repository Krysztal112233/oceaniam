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
        prelude::{Credentials, Subjects, Users},
        sea_orm_active_enums::SubjectTypeEnum,
    },
};

#[async_trait::async_trait]
pub trait UserHelper {
    /// Create user will create a [Subjects] record at same time.
    ///
    /// While create an user, at leaset put password into their [CredentialVault].
    async fn create(
        name: impl Into<String> + Send,
        credential: CredentialVault,
        application_id: Uuid,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::users::Model, Error> {
        let database = database.begin().await?;
        let result = {
            let subject =
                Subjects::create_subjects(application_id, SubjectTypeEnum::User, &database).await?;
            let _ = Credentials::create(subject.id, credential, &database).await?;

            model::users::Model {
                id: subject.id,
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

impl UserHelper for Users {}
