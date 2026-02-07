use serde_json::Value;

use crate::error::Error;

#[allow(unused)]
pub(in crate::key) trait AsSecretField {
    fn as_secret_field(&self) -> Result<Value, Error>;
}

pub(in crate::key) trait FromSecretField {
    type Type;
    fn from_secret_field(value: Value) -> Result<Self::Type, Error>;
}

pub mod rsa_key;
