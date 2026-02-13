use jsonwebtoken::jwk::Jwk;
use serde_json::Value;
use uuid::Uuid;

use crate::{error::Error, keybox::KeyOption};

#[allow(unused)]
pub(in crate::key) trait AsSecretField {
    fn as_secret_field(&self) -> Result<Value, Error>;
}

pub(in crate::key) trait FromSecretField {
    type Type;
    fn from_secret_field(value: Value) -> Result<Self::Type, Error>;
}

pub trait TryIntoKeyModel {
    fn try_into_key_model(
        self,
        application_id: Uuid,
        options: KeyOption,
    ) -> Result<oceaniam_database::model::key_boxes::Model, Error>;
}

pub trait TryIntoJwt {
    fn try_into_jwt(self) -> Result<Jwk, Error>;
}

pub mod rsa_key;
