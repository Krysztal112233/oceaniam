use std::time::Duration;

use axum::http::StatusCode;
use jsonwebtoken::Header;
use log::{debug, error};
use moka::future::{Cache, CacheBuilder};
use oceaniam_common::{
    consts,
    error::Error,
    jwks::JwkSet,
    jwt::{ClaimHelper, JwtCodec, SystemClaim},
};
use oceaniam_database::{
    helper::key_boxes::KeyBoxesHelper,
    model::{
        prelude::KeyBoxes,
        sea_orm_active_enums::{KeyAlg, KeyStatus},
    },
};
use oceaniam_keybox::{KeyBox, key::rsa_key::RsaKey};
use sea_orm::DatabaseConnection;
use tap::Tap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ManagedKeyBoxes {
    database: DatabaseConnection,
    boxes: Cache<Uuid, KeyBox>,
    banned: Cache<Uuid, ()>,
    jwks: Cache<Uuid, JwkSet>,
}

#[derive(Debug, Clone)]
pub struct SignJwtOptions {
    pub application_id: Uuid,
    pub iss: String,
    pub aud: String,
}

#[allow(unused)]
impl ManagedKeyBoxes {
    pub fn new(database: DatabaseConnection) -> Self {
        Self {
            database,
            boxes: CacheBuilder::default()
                .time_to_live(Duration::from_secs(4))
                .build(),
            banned: CacheBuilder::default()
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
                if self.banned.contains_key(&application_id) {
                    return Err(Error::with_code(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("cannot get keybox of `{application_id}`"),
                    ));
                };

                let keys = KeyBoxes::get_application_keys(application_id, &database)
                    .await
                    .inspect_err(|e| error!("{e}"))?
                    .into_iter()
                    .map(|it| (it.id, it))
                    .collect();

                let keybox = KeyBox::with_keys(application_id, keys);

                if keybox.get_keys().is_empty() {
                    self.banned.insert(application_id, ()).await;

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

    pub async fn put_keybox(&self, keybox: KeyBox) {
        self.banned.remove(&keybox.application_id()).await;
        self.boxes.insert(keybox.application_id(), keybox).await;
    }

    pub async fn sign_jwt<T>(
        self,
        sub: Uuid,
        SignJwtOptions {
            application_id,
            iss,
            aud,
        }: SignJwtOptions,
    ) -> Result<String, Error>
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

        SystemClaim::new(
            sub,
            Duration::from_hours(24 * 5).as_secs() as i64,
            Some(iss),
            Some(aud),
        )
        .encode(
            Header::new(key_alg.into()).tap_mut(|it| it.kid = Some(kid.to_string())),
            ket,
        )
        .inspect_err(|e| error!("failed to encode jwt: {}", e))
    }
}

fn h<T>(i: impl JwtCodec<T> + 'static) -> Box<dyn JwtCodec<T>>
where
    T: ClaimHelper,
{
    Box::new(i)
}
