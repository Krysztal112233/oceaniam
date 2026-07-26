use std::sync::Arc;

use crate::state::{
    applications::ManagedApplications, audit::Auditing, credentials::ManagedCredentialVaults,
    keybox::ManagedKeyBoxes, revoked::RevokedJwt,
};

use crate::error::Error;
use axum::extract::FromRef;
use oceaniam_application_secret::ApplicationSecretKeyring;
use oceaniam_auth::{
    Algorithm, Validation,
    jwks::{JwkSet, ManagedJwkSet},
    jwt::JwtValidator,
};
use oceaniam_common::config::CookieConfig;
use oceaniam_common::consts;
use oceaniam_common::crypto::MasterKey;
use oceaniam_database::{
    helper::{SafeTransactionConnectionTrait, key_boxes::KeyBoxesHelper},
    model::prelude::KeyBoxes,
};
use oceaniam_keybox::KeyBox;
use oceaniam_permission::{PermissionResolver, resolver::builtin::BuiltinResolver};
use sea_orm::DatabaseConnection;
use tap::Tap;
use tracing::{error, info, warn};

pub mod applications;
pub mod audit;
pub mod challenge;
pub mod credentials;
pub mod keybox;
pub mod revoked;
pub mod secret;

#[derive(Debug, Clone)]
pub struct AppState {
    pub database: DatabaseConnection,

    /// WARN: Only for system authentications.
    pub platform_jwks: ManagedJwkSet,

    /// WARN: Only used for system authentication validations.
    pub platform_jwt_validator: JwtValidator,

    /// System-level permission resolver for platform administrators. Application-level (per-app
    /// ORBAC) permissions are not resolved here.
    pub platform_permissions: Arc<dyn PermissionResolver + Send + Sync>,

    /// Revoked JWTs. If the system itself also uses the built-in authentication system, the related
    /// logic will also check here whether the JWT has been revoked.
    pub revoked_jwt: RevokedJwt,

    /// Keybox relative actions.
    pub keyboxes: ManagedKeyBoxes,

    /// Used for system builtin authentication and application authentication
    pub credentials: ManagedCredentialVaults,

    /// Application relative actions
    pub applications: ManagedApplications,

    pub auditing: Auditing,

    pub cookie: CookieConfig,

    pub _unit: (),
}

impl AppState {
    pub async fn new(
        database: DatabaseConnection,
        master_key: Arc<MasterKey>,
        application_secret_keyring: Arc<ApplicationSecretKeyring>,
        cookie: CookieConfig,
    ) -> Result<Self, Error> {
        let keybox = ManagedKeyBoxes::new(database.clone(), master_key.clone());

        initial_system_keybox(keybox.clone(), &database, master_key.clone()).await?;

        let credentials = ManagedCredentialVaults::new(database.clone(), master_key.clone());
        let auditing = Auditing::with_database(database.clone());
        let system_permissions = Arc::new(BuiltinResolver::new(database.clone()));

        Ok(Self {
            database: database.clone(),
            keyboxes: keybox,

            platform_jwks: initial_system_jwks(database.clone(), master_key.clone()).await?,
            platform_permissions: system_permissions,
            platform_jwt_validator: JwtValidator::new(
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
                credentials,
                database.clone(),
                auditing.clone(),
                application_secret_keyring,
            ),

            auditing,

            cookie,

            _unit: (),
        })
    }
}

async fn initial_system_jwks(
    database: DatabaseConnection,
    master_key: Arc<MasterKey>,
) -> Result<ManagedJwkSet, Error> {
    let keys = KeyBoxes::get_system_keys(&database)
        .await?
        .into_iter()
        .map(|it| (it.id, it))
        .collect();
    let system_jwks = ManagedJwkSet::new(JwkSet::from(KeyBox::with_keys(
        consts::SYSTEM_TENANT_UUID,
        keys,
        master_key,
    )));

    if system_jwks.jwks().keys.is_empty() {
        warn!("could not find any jwks, the system may not be functioning correctly.")
    }

    Ok(system_jwks)
}

async fn initial_system_keybox(
    keybox: ManagedKeyBoxes,
    database: &impl SafeTransactionConnectionTrait,
    master_key: Arc<MasterKey>,
) -> Result<(), Error> {
    match keybox.get_keybox(consts::SYSTEM_TENANT_UUID).await {
        Ok(kb) => {
            let count = kb.get_keys().len();
            info!(%count, "system keybox is ready");
        }
        Err(e) => {
            error!("could not find any system keys, creating directly: {e}");

            let mut kb = KeyBox::new(consts::SYSTEM_TENANT_UUID, master_key);
            kb.rotate().inspect_err(|e| error!("{e}"))?;

            info!(
                "the system keybox has been generated and is about to be written to the database."
            );

            kb.write_to(database).await?;

            info!("system keybox has been written to database!");
        }
    }

    Ok(())
}

impl FromRef<AppState> for () {
    fn from_ref(input: &AppState) -> Self {
        input._unit
    }
}
