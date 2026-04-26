use std::{collections::HashMap, sync::Arc};

use oceaniam_database::model::sea_orm_active_enums::ChallengeFactorType;

use crate::error::Error;

#[async_trait::async_trait]
pub trait MfaValidator: Sync + Send {
    type ValidationContext: Clone + Sync + Send;

    async fn validate(&self, ctx: Self::ValidationContext) -> Result<(), Error>;
}

#[derive(Clone)]
pub struct ValidatorRegistry<T>(
    HashMap<ChallengeFactorType, Arc<dyn MfaValidator<ValidationContext = T>>>,
);

type SharedMfaValidator<T> = Arc<dyn MfaValidator<ValidationContext = T>>;

impl<T> ValidatorRegistry<T> {
    pub fn new(input: HashMap<ChallengeFactorType, SharedMfaValidator<T>>) -> Self {
        Self(input)
    }

    pub fn get_validator(self, typ: ChallengeFactorType) -> Option<SharedMfaValidator<T>> {
        self.0.get(&typ).cloned()
    }
}

impl<T> std::fmt::Debug for ValidatorRegistry<T>
where
    T: Clone + Sync + Send,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValidatorRegistry")
            .field("factories", &self.0.keys().collect::<Vec<_>>())
            .finish()
    }
}
