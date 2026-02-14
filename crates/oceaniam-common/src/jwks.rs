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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Clone)]
pub struct ManagedJwkSet {
    jwks: Arc<RwLock<JwkSet>>,
}

impl ManagedJwkSet {
    pub async fn new(roller: impl roller::ManagedJwkSetRoller) -> Result<Self, Error> {
        Ok(Self::with_jwks(roller.roll().await?))
    }

    pub fn with_jwks(jwks: JwkSet) -> Self {
        Self {
            jwks: Arc::new(RwLock::new(jwks)),
        }
    }

    pub fn jwks(&self) -> JwkSet {
        self.jwks.read().clone()
    }
}

pub mod roller {
    use std::time::Duration;

    use log::error;
    use reqwest::{Client, StatusCode};

    use crate::{
        error::Error,
        jwks::{JwkSet, ManagedJwkSet},
    };

    #[async_trait::async_trait]
    pub trait ManagedJwkSetRoller {
        async fn roll(&self) -> Result<JwkSet, Error>;
    }

    pub struct OneShotRoller {
        url: String,
    }

    impl OneShotRoller {
        pub fn new(url: impl Into<String>) -> Self {
            Self { url: url.into() }
        }
    }

    #[async_trait::async_trait]
    impl ManagedJwkSetRoller for OneShotRoller {
        async fn roll(&self) -> Result<JwkSet, Error> {
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

    pub struct ScheduledRoller {
        url: String,
        copy: ManagedJwkSet,
    }

    impl ScheduledRoller {
        pub fn new(url: impl Into<String>, copy: ManagedJwkSet) -> Self {
            Self {
                url: url.into(),
                copy,
            }
        }
    }

    #[async_trait::async_trait]
    impl ManagedJwkSetRoller for ScheduledRoller {
        async fn roll(&self) -> Result<JwkSet, Error> {
            let copy = self.copy.clone();
            let url = self.url.clone();

            tokio::spawn(async move {
                loop {
                    // TODO: make this behavior configurable
                    tokio::time::sleep(Duration::from_mins(60)).await;

                    let jwks = OneShotRoller::new(url.clone()).roll().await;

                    if let Ok(jwks) = jwks {
                        *copy.jwks.write() = jwks
                    };
                }
            });

            OneShotRoller::new(&self.url).roll().await
        }
    }
}
