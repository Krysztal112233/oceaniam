use axum::extract::FromRef;
use im::HashMap;
use jsonwebtoken::{Algorithm, Validation};
use log::{error, info, warn};
use oceaniam_common::{consts, error::Error, jwks::ManagedJwkSet, jwt::JwtValidator};
use oceaniam_database::{
    helper::{SafeTransactionConnectionTrait, key_boxes::KeyBoxesHelper},
    model::prelude::KeyBoxes,
};
use oceaniam_keybox::{KeyBox, key::rsa_key::RsaKey};
use sea_orm::DatabaseConnection;
use tap::Tap;
use uuid::Uuid;

use crate::{
    credentials::ManagedCredentialVaults, keybox::ManagedKeyBoxes, revoked::RevokedJwt,
    roller::BuiltinScheduledJwkSetRoller, secrets::ManagedApplicationSecrets,
};

#[derive(Debug, Clone)]
pub struct AppState {
    pub database: DatabaseConnection,

    /// WARN: Only for system authentications.
    pub system_jwks: ManagedJwkSet,

    /// WARN: Only used for system authentication validations.
    pub system_jwt_validator: JwtValidator,

    /// Revoked JWTs. If the system itself also uses the built-in authentication system,
    /// the related logic will also check here whether the JWT has been revoked.
    pub revoked_jwt: RevokedJwt,

    /// Keybox relative actions.
    pub keyboxes: ManagedKeyBoxes,

    /// Used for system builtin authentication and application authentication
    pub credentials: ManagedCredentialVaults,

    /// Application's secrets relative actions
    pub application_secrets: ManagedApplicationSecrets,

    pub _unit: (),
}

impl AppState {
    pub async fn new(database: DatabaseConnection) -> Result<Self, Error> {
        let keybox = ManagedKeyBoxes::new(database.clone());

        initial_system_keybox(keybox.clone(), &database).await?;

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
            credentials: ManagedCredentialVaults::new(database.clone()),

            application_secrets: ManagedApplicationSecrets::new(database),

            _unit: (),
        })
    }
}

async fn initial_system_jwks(database: DatabaseConnection) -> Result<ManagedJwkSet, Error> {
    let roller = BuiltinScheduledJwkSetRoller::new(database);
    let system_jwks = ManagedJwkSet::with_roller(roller).await?;

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

    if keys.is_empty() || keybox.is_none() {
        info!("could not find any system keys. a new system key will be generated.");

        let mut keybox = KeyBox::new(consts::SYSTEM_APPLICATION_UUID);

        let key = RsaKey::new(
            Uuid::now_v7(),
            oceaniam_keybox::key_alg::KeyAlg::try_from(consts::SYSTEM_KEY_ALO).unwrap(),
        );
        keybox.put_key(key).inspect_err(|e| error!("{e}"))?;

        info!("the system key has been generated and is about to be written to the database.");

        keybox.clone().write_to(database).await?;

        info!("system key has been written to database!");
    }

    Ok(())
}

impl FromRef<AppState> for () {
    fn from_ref(input: &AppState) -> Self {
        input._unit
    }
}
