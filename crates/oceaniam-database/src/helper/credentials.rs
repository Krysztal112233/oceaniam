use oceaniam_common::error::Error;
use sea_orm::{ActiveModelTrait, IntoActiveModel};
use uuid::Uuid;

use crate::{
    helper::SafeTransactionConnectionTrait,
    model::{self, prelude::Credentials},
};

#[async_trait::async_trait]
pub trait CredentialsHelper {}

impl CredentialsHelper for Credentials {}
