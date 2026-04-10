use oceaniam_database::{
    helper::SafeTransactionConnectionTrait,
    model::{self, prelude::Credentials},
};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

use crate::{credential::Password, error::Error};

pub(crate) mod credential;
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

    pub fn update_password(self, password: impl AsRef<str>) -> Result<Self, Error> {
        let phc = Password::with_password(password)?.into_phc();

        #[allow(clippy::needless_update)]
        Ok(Self { phc, ..self })
    }

    pub async fn write_to(
        &self,
        id: impl Into<Uuid>,
        database: &impl SafeTransactionConnectionTrait,
    ) -> Result<model::credentials::Model, Error> {
        let Self { phc } = self;
        let id = id.into();
        let active_model = model::credentials::Model {
            id,
            phc: phc.clone(),
        }
        .into_active_model();

        let existing = Credentials::find_by_id(id)
            .one(database)
            .await
            .inspect_err(|e| error!("{e}"))?;

        Ok(match existing {
            Some(_) => Credentials::update(active_model)
                .exec(database)
                .await
                .inspect_err(|e| error!("{e}"))?,
            None => active_model
                .insert(database)
                .await
                .inspect_err(|e| error!("{e}"))?,
        })
    }
}

impl CredentialVault {
    pub async fn verify_password(&self, password: impl AsRef<str>) -> Result<bool, Error> {
        Password::from_phc(self.phc.clone())
            .verify(password.as_ref())
            .await
    }
}

impl From<model::credentials::Model> for CredentialVault {
    fn from(value: model::credentials::Model) -> Self {
        let model::credentials::Model { phc, .. } = value;

        Self { phc }
    }
}
