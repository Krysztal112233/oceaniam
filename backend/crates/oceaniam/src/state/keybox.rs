use std::time::Duration;

use axum::http::StatusCode;
use jsonwebtoken::Header;
use moka::future::{Cache, CacheBuilder};
use oceaniam_common::{
    consts,
    error::Error,
    jwks::JwkSet,
    jwt::{ClaimHelper, JwtCodec, SystemClaim},
};
use oceaniam_database::{
    helper::{
        applications::ApplicationConfiguration, applications::ApplicationHelper,
        key_boxes::KeyBoxesHelper,
    },
    model::{
        prelude::{Applications, KeyBoxes},
        sea_orm_active_enums::{KeyAlg, KeyStatus},
    },
};
use oceaniam_keybox::{KeyBox, key::rsa_key::RsaKey, keybox::KeyOption};
use sea_orm::DatabaseConnection;
use tap::Tap;
use tracing::{debug, error};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ManagedKeyBoxes {
    database: DatabaseConnection,
    boxes: Cache<Uuid, KeyBox>,
    jwks: Cache<Uuid, JwkSet>,
}

#[derive(Debug, Clone)]
pub struct SignJwtOptions {
    pub application_id: Uuid,
    pub iss: String,
    pub aud: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EncodedJwt<T> {
    pub jwt: String,
    pub claim: T,
}

#[allow(unused)]
impl ManagedKeyBoxes {
    pub fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            boxes: CacheBuilder::default()
                .time_to_live(Duration::from_secs(4))
                .build(),
            jwks: CacheBuilder::default()
                .time_to_live(Duration::from_secs(4))
                .build(),
        }
    }

    pub async fn get_keybox(&self, application_id: Uuid) -> Result<KeyBox, Error> {
        let database = self.database.clone();

        Ok(self
            .boxes
            .try_get_with::<_, Error>(application_id, async {
                let keys = KeyBoxes::get_application_keys(application_id, &database)
                    .await
                    .inspect_err(|e| error!("{e}"))?
                    .into_iter()
                    .map(|it| (it.id, it))
                    .collect();

                let keybox = KeyBox::with_keys(application_id, keys);

                if keybox.get_keys().is_empty() {
                    Err(Error::with_code(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("cannot get keybox of `{application_id}`"),
                    ))
                } else {
                    Ok(keybox)
                }
            })
            .await?)
    }

    pub async fn get_jwks(&self, application_id: Uuid) -> Result<JwkSet, Error> {
        Ok(self
            .jwks
            .try_get_with(application_id, async {
                Ok(JwkSet::from(self.clone().get_keybox(application_id).await?))
            })
            .await?)
    }

    pub async fn create_keybox(
        &self,
        application_id: Uuid,
        key_opts: KeyOption,
    ) -> Result<KeyBox, Error> {
        let mut keybox = KeyBox::new(application_id);
        keybox
            .put_key_with_option(RsaKey::new(Uuid::now_v7(), KeyAlg::Ps512), key_opts)
            .inspect_err(|e| error!("{e}"))?;
        keybox
            .write_to(&self.database)
            .await
            .inspect_err(|e| error!("{e}"))?;

        self.boxes.insert(application_id, keybox.clone()).await;

        Ok(keybox)
    }

    async fn refresh(&self, application_id: Uuid) -> Result<(), Error> {
        let keys = KeyBoxes::get_application_keys(application_id, &self.database)
            .await
            .inspect_err(|e| error!("{e}"))?
            .into_iter()
            .map(|it| (it.id, it))
            .collect();

        let keybox = KeyBox::with_keys(application_id, keys);

        self.boxes.insert(application_id, keybox).await;

        Ok(())
    }

    pub async fn sign_jwt<T>(
        self,
        sub: Uuid,
        SignJwtOptions {
            application_id,
            iss,
            aud,
        }: SignJwtOptions,
    ) -> Result<EncodedJwt<T>, Error>
    where
        T: ClaimHelper,
    {
        debug!("signing jwt for sub {}", sub);

        let Ok(keybox) = self
            .get_keybox(application_id)
            .await
            .inspect_err(|e| error!("cannot find system keybox of {application_id}: {e}"))
        else {
            return Err(Error::with_code(
                StatusCode::INTERNAL_SERVER_ERROR,
                "!!!CANNOT FIND SYSTEM KEYBOX, THIS MUST BE ERROR!!!",
            ));
        };

        let Some(key) = keybox.get_latest_raw_key(KeyStatus::Active) else {
            error!("cannot find active key in system keybox of {application_id}",);
            return Err(Error::with_code(
                StatusCode::INTERNAL_SERVER_ERROR,
                "!!!CANNOT FIND SYSTEM KEYBOX, THIS MUST BE ERROR!!!",
            ));
        };

        debug!("found active key with algorithm: {:?}", key.key_alg);

        let key_alg = key.key_alg.clone();
        let kid = key.key_id;

        let ket = match *key_alg {
            KeyAlg::Ps256
            | KeyAlg::Ps384
            | KeyAlg::Ps512
            | KeyAlg::Rs256
            | KeyAlg::Rs384
            | KeyAlg::Rs512 => h(RsaKey::try_from(key)
                .inspect_err(|e| error!("failed to convert key to rsakey: {}", e))?),
        };

        let claim = T::new(
            sub,
            Duration::from_hours(24 * 5).as_secs() as i64,
            Some(iss),
            Some(aud),
        );

        claim
            .clone()
            .encode(
                Header::new(key_alg.into()).tap_mut(|it| it.kid = Some(kid.to_string())),
                ket,
            )
            .inspect_err(|e| error!("failed to encode jwt: {}", e))
            .map(|it| EncodedJwt { jwt: it, claim })
    }

    pub async fn sign_system_jwt(self, sub: Uuid) -> Result<EncodedJwt<SystemClaim>, Error> {
        let config = {
            let model = Applications::get_system_application(&self.database).await?;
            serde_json::from_value::<ApplicationConfiguration>(model.configuration)
        }?;

        self.sign_jwt(
            sub,
            SignJwtOptions {
                application_id: consts::SYSTEM_APPLICATION_UUID,
                iss: config.auth.token.issuer,
                aud: config.auth.token.audience,
            },
        )
        .await
    }
}

fn h<T>(i: impl JwtCodec<T> + 'static) -> Box<dyn JwtCodec<T>>
where
    T: ClaimHelper,
{
    Box::new(i)
}
