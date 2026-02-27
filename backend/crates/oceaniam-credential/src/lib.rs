use log::error;
use oceaniam_database::{
    helper::SafeTransactionConnectionTrait,
    model::{self, prelude::Credentials},
};
use sea_orm::{EntityTrait, IntoActiveModel};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{credential::Password, error::Error};

pub mod credential;
pub mod error;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CredentialVault {
    /// This field used to store [PHC](argon2::PasswordHash) string.
    pub phc: String,
}

impl CredentialVault {
    /// User must have at least password login method enabled.
    pub fn with_password(password: impl AsRef<str>) -> Result<Self, Error> {
        let phc = Password::with_password(password)?.into_phc();

        Ok(Self { phc })
    }

    pub async fn write_to(
        &self,
        id: impl Into<Uuid>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::credentials::Model, Error> {
        let Self { phc } = self;

        Ok(Credentials::update(
            model::credentials::Model {
                id: id.into(),
                phc: phc.clone(),
            }
            .into_active_model(),
        )
        .exec(database)
        .await
        .inspect_err(|e| error!("{e}"))?)
    }
}

impl From<model::credentials::Model> for CredentialVault {
    fn from(value: model::credentials::Model) -> Self {
        let model::credentials::Model { phc, .. } = value;

        Self { phc }
    }
}
