use jsonwebtoken::Algorithm;
use oceaniam_database::model::sea_orm_active_enums::KeyAlg as DatabaseKeyAlgorithm;
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(transparent)]
pub struct KeyAlg(DatabaseKeyAlgorithm);

macro_rules! alg_conversions {
    ($(($jwt:ident, $db:ident)),+ $(,)?) => {
        impl TryFrom<Algorithm> for KeyAlg {
            type Error = Error;

            fn try_from(value: Algorithm) -> Result<Self, Self::Error> {
                match value {
                    $(Algorithm::$jwt => Ok(Self(DatabaseKeyAlgorithm::$db)),)+
                    _ => Err(Error::unimplemented_jwt_alogrithm(value)),
                }
            }
        }

        impl From<KeyAlg> for Algorithm {
            fn from(value: KeyAlg) -> Self {
                match value.0 {
                    $(DatabaseKeyAlgorithm::$db => Algorithm::$jwt),+
                }
            }
        }
    };
}

alg_conversions!(
    (RS256, Rs256),
    (RS384, Rs384),
    (RS512, Rs512),
    (PS256, Ps256),
    (PS384, Ps384),
    (PS512, Ps512),
);

impl From<DatabaseKeyAlgorithm> for KeyAlg {
    fn from(value: DatabaseKeyAlgorithm) -> Self {
        Self(value)
    }
}

impl From<KeyAlg> for DatabaseKeyAlgorithm {
    fn from(value: KeyAlg) -> Self {
        value.0
    }
}

impl std::ops::Deref for KeyAlg {
    type Target = DatabaseKeyAlgorithm;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
