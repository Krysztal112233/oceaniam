use jsonwebtoken::Algorithm;
use oceaniam_database::model::sea_orm_active_enums::KeyAlg as InnerKeyAlg;
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(transparent)]
pub struct KeyAlg(InnerKeyAlg);

impl TryFrom<Algorithm> for KeyAlg {
    type Error = crate::error::Error;

    fn try_from(value: Algorithm) -> Result<Self, Self::Error> {
        let inner = match value {
            Algorithm::RS256 => InnerKeyAlg::Rs256,
            Algorithm::RS384 => InnerKeyAlg::Rs384,
            Algorithm::RS512 => InnerKeyAlg::Rs512,
            Algorithm::PS256 => InnerKeyAlg::Ps256,
            Algorithm::PS384 => InnerKeyAlg::Ps384,
            Algorithm::PS512 => InnerKeyAlg::Ps512,
            _ => return Err(Error::unimplemented_jwt_alogrithm(value)),
        };

        Ok(KeyAlg(inner))
    }
}

impl From<InnerKeyAlg> for KeyAlg {
    fn from(value: InnerKeyAlg) -> Self {
        Self(value)
    }
}

impl From<KeyAlg> for InnerKeyAlg {
    fn from(value: KeyAlg) -> Self {
        value.0
    }
}

impl From<KeyAlg> for Algorithm {
    fn from(value: KeyAlg) -> Self {
        match value.0 {
            InnerKeyAlg::Rs256 => Algorithm::RS256,
            InnerKeyAlg::Rs384 => Algorithm::RS384,
            InnerKeyAlg::Rs512 => Algorithm::RS512,
            InnerKeyAlg::Ps256 => Algorithm::PS256,
            InnerKeyAlg::Ps384 => Algorithm::PS384,
            InnerKeyAlg::Ps512 => Algorithm::PS512,
        }
    }
}

impl std::ops::Deref for KeyAlg {
    type Target = InnerKeyAlg;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
