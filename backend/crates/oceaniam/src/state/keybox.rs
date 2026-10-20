use std::sync::Arc;
use std::time::Duration;

use crate::error::Error;
use axum::http::StatusCode;
use moka::future::{Cache, CacheBuilder};
use oceaniam_auth::{
    Header,
    jwks::JwkSet,
    jwt::{ClaimHelper, JwtCodec, SystemClaim},
};
use oceaniam_common::{consts, crypto::MasterKey, run_cpu_bound};
use oceaniam_database::{
    config::application::ApplicationConfiguration,
    helper::{
        SafeTransactionConnectionTrait, applications::ApplicationHelper, db_clock_now,
        key_boxes::KeyBoxesHelper, tenants::TenantsHelper,
    },
    model::{
        prelude::{Applications, KeyBoxes, Tenants},
        sea_orm_active_enums::{KeyAlg, KeyStatus},
    },
};
use oceaniam_keybox::{KeyBox, RawKey, RsaKey};
use sea_orm::DatabaseConnection;
use serde::{Serialize, de::DeserializeOwned};
use tap::Tap;
use tracing::{debug, error, field};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ManagedKeyBoxes {
    database: DatabaseConnection,

    master_key: Arc<MasterKey>,

    boxes: Cache<Uuid, KeyBox>,

    jwks: Cache<Uuid, JwkSet>,
}

#[derive(Debug, Clone)]
pub struct SignJwtOptions {
    pub tenant_id: Uuid,
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
    pub fn new(database: DatabaseConnection, master_key: Arc<MasterKey>) -> Self {
        Self {
            database,
            master_key,
            boxes: CacheBuilder::default()
                .time_to_live(Duration::from_secs(4))
                .build(),
            jwks: CacheBuilder::default()
                .time_to_live(Duration::from_secs(4))
                .build(),
        }
    }

    #[tracing::instrument(
        level = "info",
        name = "keybox.get_keybox",
        skip_all,
        fields(otel.kind = "internal")
    )]
    pub async fn get_keybox(&self, tenant_id: Uuid) -> Result<KeyBox, Error> {
        let database = self.database.clone();
        let master_key = self.master_key.clone();

        Ok(self
            .boxes
            .try_get_with::<_, Error>(tenant_id, async move {
                let keys = KeyBoxes::get_tenant_keys(tenant_id, &database)
                    .await
                    .inspect_err(|e| error!("{e}"))?
                    .into_iter()
                    .map(|it| (it.id, it))
                    .collect();

                let keybox = KeyBox::with_keys(tenant_id, keys, master_key.clone());

                if keybox.get_keys().is_empty() {
                    // Absent tenants used to reach RSA generation here and fail only on the
                    // `key_boxes` FK afterwards, giving anonymous callers a CPU amplification
                    // vector. Verify the tenant row exists before generating anything; the
                    // probe deliberately includes the system tenant so the legacy system JWKS
                    // route keeps its auto-bootstrap behavior.
                    let tenant_exists = Tenants::tenant_exists(tenant_id, &database)
                        .await
                        .inspect_err(
                            |e| error!(%tenant_id, error = %e, "failed to check tenant existence"),
                        )?;
                    if !tenant_exists {
                        debug!(%tenant_id, "refusing to auto-create keybox for absent tenant");
                        return Err(Error::with_code(StatusCode::NOT_FOUND, "tenant not found"));
                    }

                    debug!(%tenant_id, "keybox is empty, auto-creating default keybox");
                    let now = db_clock_now(&database)
                        .await
                        .inspect_err(|e| error!("{e}"))?;
                    let mut keybox = KeyBox::new(tenant_id, master_key);
                    keybox.rotate(now).await.inspect_err(|e| error!("{e}"))?;
                    keybox
                        .write_to(&database)
                        .await
                        .inspect_err(|e| error!("{e}"))?;
                    Ok(keybox)
                } else {
                    Ok(keybox)
                }
            })
            .await?)
    }

    #[tracing::instrument(
        level = "info",
        name = "keybox.get_jwks",
        skip_all,
        fields(otel.kind = "internal")
    )]
    pub async fn get_jwks(&self, tenant_id: Uuid) -> Result<JwkSet, Error> {
        Ok(self
            .jwks
            .try_get_with(tenant_id, async {
                Ok(JwkSet::try_from(self.clone().get_keybox(tenant_id).await?)?)
            })
            .await?)
    }

    /// Creates a new initial keybox for a tenant with a single initial key.
    ///
    /// This is called during tenant creation to bootstrap the tenant's
    /// signing key.  The initial key is created with [`KeyOption`] timestamps anchored on a
    /// database-clock sample and immediately persisted to the database and cached.
    pub async fn create_keybox(&self, tenant_id: Uuid) -> Result<KeyBox, Error> {
        let keybox = self.create_keybox_in_tx(tenant_id, &self.database).await?;

        self.boxes.insert(tenant_id, keybox.clone()).await;

        Ok(keybox)
    }

    #[tracing::instrument(
        level = "info",
        name = "keybox.create_keybox_in_tx",
        skip_all,
        fields(otel.kind = "internal")
    )]
    pub async fn create_keybox_in_tx(
        &self,
        tenant_id: Uuid,
        transaction: &impl SafeTransactionConnectionTrait,
    ) -> Result<KeyBox, Error> {
        let mut keybox = KeyBox::new(tenant_id, self.master_key.clone());

        let now = db_clock_now(transaction).await?;
        keybox.rotate(now).await.inspect_err(|e| error!("{e}"))?;

        keybox
            .write_to(transaction)
            .await
            .inspect_err(|e| error!("{e}"))?;

        Ok(keybox)
    }

    pub async fn insert_cache(&self, tenant_id: Uuid, keybox: KeyBox) {
        self.boxes.insert(tenant_id, keybox).await;
    }

    pub async fn invalidate(&self, tenant_id: Uuid) {
        self.boxes.invalidate(&tenant_id).await;
        self.jwks.invalidate(&tenant_id).await;
    }

    /// Ensures the tenant's keybox has at least one Active and one Pending
    /// key, then persists and invalidates caches so subsequent requests pick
    /// up the new state immediately.
    ///
    /// Delegates the invariant enforcement to [`KeyBox::rotate`].
    #[tracing::instrument(
        level = "info",
        name = "keybox.rotate_key",
        skip_all,
        fields(otel.kind = "internal")
    )]
    pub async fn rotate_key(&self, tenant_id: Uuid) -> Result<(), Error> {
        let mut keybox = self.get_keybox(tenant_id).await?;
        let now = db_clock_now(&self.database).await?;
        keybox.rotate(now).await?;

        keybox.write_to(&self.database).await?;

        self.boxes.insert(tenant_id, keybox).await;
        self.jwks.invalidate(&tenant_id).await;

        Ok(())
    }

    #[tracing::instrument(
        level = "info",
        name = "keybox.revoke_key",
        skip_all,
        fields(otel.kind = "internal")
    )]
    pub async fn revoke_key(&self, tenant_id: Uuid, key_id: Uuid) -> Result<(), Error> {
        let mut keybox = self.get_keybox(tenant_id).await?;

        let now = db_clock_now(&self.database).await?;
        keybox.revoke_key(&key_id, now)?;
        keybox.write_to(&self.database).await?;

        self.boxes.insert(tenant_id, keybox).await;
        self.jwks.invalidate(&tenant_id).await;

        Ok(())
    }

    #[tracing::instrument(
        level = "info",
        name = "keybox.sign_jwt",
        skip_all,
        fields(otel.kind = "internal")
    )]
    pub async fn sign_jwt<T>(
        self,
        sub: Uuid,
        SignJwtOptions {
            tenant_id,
            iss,
            aud,
        }: SignJwtOptions,
    ) -> Result<EncodedJwt<T>, Error>
    where
        T: ClaimHelper + Send + 'static,
    {
        debug!("signing jwt for sub {}", sub);

        let key = self.active_signing_key(tenant_id).await?;

        let claim = T::new(
            sub,
            Duration::from_hours(24 * 5).as_secs() as i64,
            Some(iss),
            Some(aud),
        );

        sign_claim_with_raw_key(key, self.master_key, claim).await
    }

    /// Loads the tenant keybox and selects its latest [`Active`](KeyStatus::Active) signing key.
    ///
    /// Shared by [`Self::sign_jwt`] and [`Self::sign_assembled_jwt`] so both paths keep identical
    /// key selection and failure semantics (500 on a missing keybox or missing active key).
    async fn active_signing_key(&self, tenant_id: Uuid) -> Result<RawKey, Error> {
        let Ok(keybox) = self
            .get_keybox(tenant_id)
            .await
            .inspect_err(|e| error!("cannot find keybox of {tenant_id}: {e}"))
        else {
            return Err(Error::with_code(
                StatusCode::INTERNAL_SERVER_ERROR,
                "!!!CANNOT FIND TENANT KEYBOX, THIS MUST BE ERROR!!!",
            ));
        };

        let Some(key) = keybox.get_latest_raw_key(KeyStatus::Active) else {
            error!("cannot find active key in keybox of {tenant_id}",);
            return Err(Error::with_code(
                StatusCode::INTERNAL_SERVER_ERROR,
                "!!!CANNOT FIND ACTIVE KEY IN TENANT KEYBOX, THIS MUST BE ERROR!!!",
            ));
        };

        debug!("found active key with algorithm: {:?}", key.key_alg);

        Ok(key)
    }

    /// Signs an already-assembled claim set with the tenant keybox's active key.
    ///
    /// OIDC token issuance needs claim shapes (`nonce`/`auth_time`, no `jti`) and a fixed
    /// one-hour TTL that the hardcoded five-day `ClaimHelper::new` construction in
    /// [`Self::sign_jwt`] cannot express, so the caller assembles the claims. Key selection
    /// and failure semantics are identical to `sign_jwt`, which is untouched.
    ///
    /// Note: callers invoke this while holding a database transaction with row locks (OIDC
    /// token issuance), so the lock is held across the RSA signing and, on a keybox cache
    /// miss, across the keybox read as well — a throughput consideration when sizing the
    /// transaction scope.
    #[tracing::instrument(
        level = "info",
        name = "keybox.sign_assembled_jwt",
        skip_all,
        fields(otel.kind = "internal")
    )]
    pub async fn sign_assembled_jwt<T>(
        self,
        tenant_id: Uuid,
        claim: T,
    ) -> Result<EncodedJwt<T>, Error>
    where
        T: Serialize + DeserializeOwned + Clone + Send + 'static,
    {
        let key = self.active_signing_key(tenant_id).await?;

        sign_claim_with_raw_key(key, self.master_key, claim).await
    }

    #[tracing::instrument(
        level = "info",
        name = "keybox.sign_system_jwt",
        skip_all,
        fields(otel.kind = "internal")
    )]
    pub async fn sign_system_jwt(self, sub: Uuid) -> Result<EncodedJwt<SystemClaim>, Error> {
        let config = {
            let model = Applications::get_system_application(&self.database).await?;
            serde_json::from_value::<ApplicationConfiguration>(model.configuration)
        }?;

        self.sign_jwt(
            sub,
            SignJwtOptions {
                tenant_id: consts::SYSTEM_TENANT_UUID,
                iss: config.auth.token.issuer,
                aud: config.auth.token.audience,
            },
        )
        .await
    }
}

/// Shared RSA signing body. `ClaimHelper::encode` is exactly `JwtCodec::encode`, so routing
/// `ClaimHelper` claims through this general path is behavior-identical, and claim shapes
/// outside the `ClaimHelper` contract (OIDC ID tokens carry `nonce`/`auth_time` and no `jti`)
/// can be signed as well.
async fn sign_claim_with_raw_key<T>(
    key: RawKey,
    master_key: Arc<MasterKey>,
    claim: T,
) -> Result<EncodedJwt<T>, Error>
where
    T: Serialize + DeserializeOwned + Clone + Send + 'static,
{
    let key_alg = key.key_alg.clone();
    let kid = key.key_id;
    let queue_span = tracing::info_span!(
        "keybox.rsa.sign.queue",
        otel.kind = "internal",
        cpu.operation = "rsa.sign_jwt",
        key.id = %kid,
        rsa.algorithm = ?key_alg,
    );

    run_cpu_bound(queue_span, move |parent| {
        let span = tracing::info_span!(
            parent: &parent,
            "keybox.rsa.sign",
            otel.kind = "internal",
            otel.status_code = field::Empty,
            otel.status_description = field::Empty,
            cpu.operation = "rsa.sign_jwt",
            key.id = %kid,
            rsa.algorithm = ?key_alg,
        );
        let result = span.in_scope(|| {
            let rsa_key = match *key_alg {
                KeyAlg::Ps256
                | KeyAlg::Ps384
                | KeyAlg::Ps512
                | KeyAlg::Rs256
                | KeyAlg::Rs384
                | KeyAlg::Rs512 => h(RsaKey::from_raw_key(key, &master_key)
                    .inspect_err(|e| error!(error = %e, "failed to unseal RSA signing key"))?),
            };
            let jwt = rsa_key
                .encode(
                    Header::new(key_alg.into()).tap_mut(|it| it.kid = Some(kid.to_string())),
                    claim.clone(),
                )
                .inspect_err(|e| error!(error = %e, "failed to sign JWT"))?;

            Ok(EncodedJwt { jwt, claim })
        });

        if result.is_err() {
            span.record("otel.status_code", "ERROR");
            span.record("otel.status_description", "RSA JWT signing failed");
        }

        result
    })
    .await
    .map_err(|source| Error::Internal {
        msg: format!("CPU-bound RSA JWT signing task failed: {source}"),
        location: snafu::location!(),
    })?
}

fn h<T>(i: impl JwtCodec<T> + 'static) -> Box<dyn JwtCodec<T>>
where
    T: DeserializeOwned + Serialize,
{
    Box::new(i)
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};
    use std::sync::Mutex;

    use oceaniam_auth::{Algorithm, TokenData, Validation, jwt::SystemClaim};
    use tracing::Instrument as _;
    use tracing_subscriber::fmt::format::FmtSpan;

    use super::*;

    #[derive(Clone)]
    struct Buffer(Arc<Mutex<Vec<u8>>>);

    impl Write for Buffer {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0
                .lock()
                .expect("trace buffer lock")
                .extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    // NOTE: AI-generated test
    #[tokio::test]
    async fn rsa_jwt_signing_spans_cross_blocking_dispatch_without_token() {
        let master_key_hex = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let master_key = Arc::new(MasterKey::from_hex(master_key_hex).expect("test master key"));
        let subject = Uuid::now_v7();
        let key = RsaKey::with_bit_size(Uuid::now_v7(), KeyAlg::Rs256, 2048)
            .expect("generate test RSA key");
        let raw_key = key
            .clone()
            .into_raw_key(&master_key)
            .expect("seal test RSA key");
        let claim = SystemClaim::new(subject, 60, None, None);

        let bytes = Arc::new(Mutex::new(Vec::new()));
        let writer = bytes.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(move || Buffer(writer.clone()))
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .finish();
        let dispatch = tracing::Dispatch::new(subscriber);
        let _default = tracing::dispatcher::set_default(&dispatch);

        let encoded = async { sign_claim_with_raw_key(raw_key, master_key, claim).await }
            .instrument(tracing::info_span!("test.request"))
            .await
            .expect("sign test JWT");
        let decoded: TokenData<SystemClaim> = key
            .decode(encoded.jwt.as_bytes(), &Validation::new(Algorithm::RS256))
            .expect("decode signed JWT");
        assert_eq!(decoded.claims.sub, subject);

        let output = String::from_utf8(bytes.lock().expect("trace buffer lock").clone())
            .expect("utf8 trace output");
        assert!(output.contains("keybox.rsa.sign.queue"));
        assert!(output.contains("keybox.rsa.sign"));
        assert!(output.contains("keybox.private_key.unseal"));
        assert!(output.contains("auth.jwt.encode"));
        assert!(output.contains("test.request"));
        assert!(!output.contains(&encoded.jwt));
        assert!(!output.contains(master_key_hex));
    }
}
