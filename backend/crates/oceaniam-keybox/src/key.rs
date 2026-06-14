use oceaniam_auth::jwks::Jwk;
use oceaniam_common::crypto::MasterKey;
use serde_json::Value;
use uuid::Uuid;

use crate::{error::Error, keybox::KeyOption};

pub(in crate::key) trait FromSecretField {
    type Type;
    fn from_secret_field(value: Value, master_key: &MasterKey) -> Result<Self::Type, Error>;
}

pub trait TryIntoKeyModel {
    fn try_into_key_model(
        self,
        tenant_id: Uuid,
        master_key: &MasterKey,
        options: KeyOption,
    ) -> Result<oceaniam_database::model::key_boxes::Model, Error>;
}

pub trait TryIntoJwk {
    fn try_into_jwk(self) -> Result<Jwk, Error>;
}

pub mod rsa_key;
