use oceaniam_common::error::Error;
use oceaniam_credential::CredentialVault;
use sea_orm::{ActiveModelTrait, IntoActiveModel};
use uuid::Uuid;

use crate::{
    helper::SafeTransactionConnectionTrait,
    model::{self, prelude::Credentials},
};

#[async_trait::async_trait]
pub trait CredentialsHelper {
    async fn create(
        id: Uuid,
        credential: CredentialVault,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::credentials::Model, Error> {
        let CredentialVault { phc } = credential;

        let credential = model::credentials::Model { id, phc }
            .into_active_model()
            .insert(database)
            .await?;

        Ok(credential)
    }
}

impl CredentialsHelper for Credentials {}
