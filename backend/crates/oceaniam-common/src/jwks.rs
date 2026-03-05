use std::sync::Arc;

use im::Vector;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::error::Error;

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct Jwk {
    pub kty: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub kid: Option<String>,

    #[serde(rename = "use", skip_serializing_if = "Option::is_none")]
    pub use_: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub alg: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub e: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct JwkSet {
    pub keys: Vector<Jwk>,
}

/// NOTE: THIS STRUCT JUST FOR SIMPLE SCHEMA. DONT CONSTRUCT OR USE IT.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct JwkSetSchema {
    pub keys: Vec<Jwk>,
}

// TODO: OPTIMIZE here
impl From<JwkSet> for jsonwebtoken::jwk::JwkSet {
    fn from(value: JwkSet) -> Self {
        let value = serde_json::to_value(value).unwrap();

        serde_json::from_value(value).unwrap()
    }
}

// TODO: OPTIMIZE here
impl From<Jwk> for jsonwebtoken::jwk::Jwk {
    fn from(value: Jwk) -> Self {
        let value = serde_json::to_value(value).unwrap();

        serde_json::from_value(value).unwrap()
    }
}

// TODO: OPTIMIZE here
impl From<jsonwebtoken::jwk::Jwk> for Jwk {
    fn from(value: jsonwebtoken::jwk::Jwk) -> Self {
        let value = serde_json::to_value(value).unwrap();

        serde_json::from_value(value).unwrap()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ManagedJwkSet {
    jwks: Arc<RwLock<JwkSet>>,
}

impl ManagedJwkSet {
    pub async fn with_roller(roller: impl roller::ManagedJwkSetRoller) -> Result<Self, Error> {
        let jwks = Self::default();
        roller.roll(jwks.clone()).await?;
        Ok(jwks)
    }

    pub fn jwks(&self) -> JwkSet {
        self.jwks.read().clone()
    }

    pub fn set_jwks(&mut self, jwks: JwkSet) {
        *self.jwks.write() = jwks;
    }
}

pub mod roller {
    use std::time::Duration;

    use reqwest::{Client, StatusCode};
    use tracing::error;

    use crate::{
        error::Error,
        jwks::{JwkSet, ManagedJwkSet},
    };

    #[async_trait::async_trait]
    pub trait ManagedJwkSetRoller {
        async fn roll(&self, copy: ManagedJwkSet) -> Result<(), Error>;
    }

    pub struct OneShotRoller {
        url: String,
    }

    impl OneShotRoller {
        pub fn new(url: impl Into<String>) -> Self {
            Self { url: url.into() }
        }

        pub async fn pull(&self) -> Result<JwkSet, Error> {
            Client::new()
                .get(self.url.clone())
                .send()
                .await
                .inspect_err(|e| error!("{e}"))
                .map_err(|e| Error::with_code(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                .json()
                .await
                .inspect_err(|e| error!("{e}"))
                .map_err(|e| Error::with_code(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }

    #[async_trait::async_trait]
    impl ManagedJwkSetRoller for OneShotRoller {
        async fn roll(&self, copy: ManagedJwkSet) -> Result<(), Error> {
            let jwks = self.pull().await?;
            *copy.jwks.write() = jwks;

            Ok(())
        }
    }

    pub struct ScheduledRoller {
        url: String,
    }

    impl ScheduledRoller {
        pub fn new(url: impl Into<String>) -> Self {
            Self { url: url.into() }
        }
    }

    #[async_trait::async_trait]
    impl ManagedJwkSetRoller for ScheduledRoller {
        async fn roll(&self, copy: ManagedJwkSet) -> Result<(), Error> {
            let copy = copy.clone();
            let clond = copy.clone();
            let url = self.url.clone();

            tokio::spawn(async move {
                loop {
                    // TODO: make this behavior configurable
                    tokio::time::sleep(Duration::from_mins(60)).await;

                    let _ = OneShotRoller::new(url.clone()).roll(clond.clone()).await;
                }
            });

            OneShotRoller::new(&self.url).roll(copy.clone()).await
        }
    }
}
