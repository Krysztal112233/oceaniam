use crate::model::prelude::Credentials;

#[async_trait::async_trait]
pub trait CredentialsHelper {}

impl CredentialsHelper for Credentials {}
