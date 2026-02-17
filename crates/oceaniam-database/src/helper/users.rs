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
pub trait UserHelper {}

impl UserHelper for Users {}
