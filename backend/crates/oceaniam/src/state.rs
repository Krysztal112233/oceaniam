use crate::state::{
    applications::ManagedApplications, audit::Auditing, credentials::ManagedCredentialVaults,
    filters::ManagedFilters, keybox::ManagedKeyBoxes, revoked::RevokedJwt,
};

use axum::extract::FromRef;
use im::HashMap;
use oceaniam_auth::{
    Algorithm, Validation,
    jwks::{JwkSet, ManagedJwkSet},
    jwt::JwtValidator,
};
use oceaniam_common::{consts, error::Error};
use oceaniam_database::{
    helper::{SafeTransactionConnectionTrait, key_boxes::KeyBoxesHelper},
    model::prelude::KeyBoxes,
};
use oceaniam_keybox::{KeyBox, key::rsa_key::RsaKey};
use sea_orm::DatabaseConnection;
use tap::Tap;
use tracing::{error, info, warn};
use uuid::Uuid;

pub mod applications;
pub mod audit;
pub mod challenge;
pub mod credentials;
pub mod filters;
pub mod keybox;
pub mod revoked;
pub mod secret;

#[derive(Debug, Clone)]
pub struct AppState<'a> {
    pub database: DatabaseConnection,

    /// WARN: Only for system authentications.
    pub system_jwks: ManagedJwkSet,

    /// WARN: Only used for system authentication validations.
    pub system_jwt_validator: JwtValidator,

    /// Revoked JWTs. If the system itself also uses the built-in authentication system, the related
    /// logic will also check here whether the JWT has been revoked.
    pub revoked_jwt: RevokedJwt,

    /// Keybox relative actions.
    pub keyboxes: ManagedKeyBoxes,

    /// Used for system builtin authentication and application authentication
    pub credentials: ManagedCredentialVaults,

    /// Application relative actions
    pub applications: ManagedApplications<'a>,

    pub filters: ManagedFilters<'a>,

    pub auditing: Auditing,

    pub _unit: (),
}

impl AppState<'static> {
    pub async fn new(database: DatabaseConnection) -> Result<Self, Error> {
        let keybox = ManagedKeyBoxes::new(database.clone());

        initial_system_keybox(keybox.clone(), &database).await?;

        let credentials = ManagedCredentialVaults::new(database.clone());
        let filters = ManagedFilters::new(database.clone());
        let auditing = Auditing::with_database(database.clone());
        Ok(Self {
            database: database.clone(),
            keyboxes: keybox,
            system_jwks: initial_system_jwks(database.clone()).await?,

            system_jwt_validator: JwtValidator::new(
                Validation::default()
                    .tap_mut(|it| it.set_audience(&["OceanIAM"]))
                    .tap_mut(|it| {
                        it.algorithms = vec![
                            Algorithm::PS256,
                            Algorithm::PS384,
                            Algorithm::PS512,
                            Algorithm::RS256,
                            Algorithm::RS384,
                            Algorithm::RS512,
                        ]
                    }),
            ),
            revoked_jwt: RevokedJwt::new(database.clone()),
            credentials: credentials.clone(),

            applications: ManagedApplications::new(
                filters.clone(),
                credentials,
                database.clone(),
                auditing.clone(),
            ),

            auditing,

            filters,

            _unit: (),
        })
    }
}

async fn initial_system_jwks(database: DatabaseConnection) -> Result<ManagedJwkSet, Error> {
    let keys = KeyBoxes::get_system_keys(&database)
        .await?
        .into_iter()
        .map(|it| (it.id, it))
        .collect();
    let system_jwks = ManagedJwkSet::new(JwkSet::from(KeyBox::with_keys(
        consts::SYSTEM_APPLICATION_UUID,
        keys,
    )));

    if system_jwks.jwks().keys.is_empty() {
        warn!("could not find any jwks, the system may not be functioning correctly.")
    }

    Ok(system_jwks)
}

async fn initial_system_keybox(
    keybox: ManagedKeyBoxes,
    database: &impl SafeTransactionConnectionTrait,
) -> Result<(), Error> {
    let keys: HashMap<_, _> = KeyBoxes::get_system_keys(database)
        .await?
        .into_iter()
        .map(|it| (it.id, it))
        .collect();

    let keybox = keybox.get_keybox(consts::SYSTEM_APPLICATION_UUID).await;

    if keys.is_empty() || keybox.is_err() {
        info!("could not find any system keys. a new system key will be generated.");

        let mut keybox = KeyBox::new(consts::SYSTEM_APPLICATION_UUID);

        let key = RsaKey::new(
            Uuid::now_v7(),
            oceaniam_keybox::key_alg::KeyAlg::try_from(consts::SYSTEM_KEY_ALO).unwrap(),
        );
        keybox.add_key(key).inspect_err(|e| error!("{e}"))?;

        info!("the system key has been generated and is about to be written to the database.");

        keybox.clone().write_to(database).await?;

        info!("system key has been written to database!");
    }

    Ok(())
}

impl FromRef<AppState<'_>> for () {
    fn from_ref(input: &AppState) -> Self {
        input._unit
    }
}
