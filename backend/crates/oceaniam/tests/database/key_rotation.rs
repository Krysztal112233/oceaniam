//! Transition test for the tenant signing algorithm migration: a legacy PS512-only tenant
//! fleet converges to RS256 (`TENANT_KEY_ALG`) through a real `key_rotation` worker run, and
//! both the discovery advertisement and newly issued ID tokens follow the live key state.

use std::sync::Arc;

use chrono::{DateTime, Duration, FixedOffset, Utc};
use oceaniam_common::{consts::WORKER_ID_KEY_ROTATION, crypto::MasterKey};
use oceaniam_database::{
    helper::key_boxes::KeyBoxesHelper,
    model::{
        key_boxes::{self, Column},
        prelude::KeyBoxes,
        sea_orm_active_enums::{KeyAlg, KeyStatus},
    },
};
use oceaniam_keybox::{KeyOption, RsaKey, TryIntoKeyModel};
use oceaniam_worker::{WorkerContext, collect_workers};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
};
use uuid::Uuid;

use crate::support::{
    TestApp,
    oidc_drive::{drive_user_token_response, seed_login_fixture, spawn_preview},
    sqid_to_uuid,
};

/// Deterministic non-zero test KEK — not secret, tests only. The same fixed value as
/// `tests/support/mod.rs::TEST_MASTER_KEY_HEX`, kept private there.
const TEST_MASTER_KEY_HEX: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn test_master_key() -> MasterKey {
    MasterKey::from_hex(TEST_MASTER_KEY_HEX).expect("test master key")
}

/// Replaces the tenant's bootstrapped keybox with a legacy fleet: a single PS512 key stored
/// as `Active` whose lifecycle timestamps already lie in the past, so the worker's status
/// refresh retires it and the rotation generates a replacement ACTIVE key. The stored status
/// still reads `Active`, which is exactly the pre-rotation truth a legacy fleet carries.
async fn seed_ps512_only_keybox(app: &TestApp, tenant_id: Uuid) {
    let database = app.database().await;
    KeyBoxes::delete_many()
        .filter(Column::TenantId.eq(tenant_id))
        .exec(&database)
        .await
        .expect("delete bootstrapped tenant keys");

    let now: DateTime<FixedOffset> = Utc::now().fixed_offset();
    let activated_at = now - Duration::days(61);
    let options = KeyOption {
        created_at: activated_at,
        activated_at,
        retired_at: now - Duration::days(31),
        expires_at: now - Duration::days(1),
    };
    let key = RsaKey::with_bit_size(Uuid::now_v7(), KeyAlg::Ps512, 2048)
        .expect("generate PS512 seed key")
        // Anchor the status computation at activation so the row is stored as Active.
        .try_into_key_model(tenant_id, &test_master_key(), options, activated_at)
        .expect("seal PS512 seed key");
    key.into_active_model()
        .insert(&database)
        .await
        .expect("insert PS512 seed key");

    // Tenant creation cached the bootstrapped keybox; drop it so the next read sees the
    // seeded fleet.
    app.state.keyboxes.invalidate(tenant_id).await;
}

async fn advertised_algs(app: &TestApp, tenant_sqid: &str) -> Vec<String> {
    let response = app
        .client
        .get(app.url(&format!(
            "/oidc/{tenant_sqid}/.well-known/openid-configuration"
        )))
        .send()
        .await
        .expect("discovery request failed");
    assert_eq!(response.status(), 200, "discovery returns 200");
    let body: serde_json::Value = response.json().await.expect("discovery body");
    serde_json::from_value(body["id_token_signing_alg_values_supported"].clone())
        .expect("advertised alg list is a string array")
}

/// Drives the registered `key_rotation` worker once, like the `oidc_expired_cleanup` tests
/// drive their worker through `collect_workers`.
async fn run_key_rotation_once(app: &TestApp) {
    let context = WorkerContext {
        database: app.database().await,
        master_key: Arc::new(test_master_key()),
    };
    let workers = collect_workers();
    let worker = workers
        .get(WORKER_ID_KEY_ROTATION)
        .expect("key_rotation worker must be registered");
    worker
        .worker
        .run(&context)
        .await
        .expect("key rotation run succeeds");
}

/// G2/G4/G5 transition pin: a PS512-only fleet rotates into RS256 signing keys, the
/// discovery advertisement follows the live key state, and an ID token issued after the
/// rotation is signed with RS256.
// NOTE: AI-generated test
#[tokio::test]
async fn ps512_only_fleet_converges_to_rs256_through_rotation() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let tenant_id = sqid_to_uuid(&fixture.tenant_sqid);
    seed_ps512_only_keybox(&app, tenant_id).await;

    // Pre-rotation truth: the stored fleet is PS512-only, so discovery advertises PS512.
    assert_eq!(
        advertised_algs(&app, &fixture.tenant_sqid).await,
        ["PS512"],
        "pre-rotation discovery must advertise the extant PS512 fleet"
    );

    // The seeded key's expires_at lies in the past, so the worker retires it, finds no active
    // key, and rotates — the "no active key" trigger of key_rotation.rs.
    run_key_rotation_once(&app).await;
    // The worker writes straight to the database behind the app's moka caches.
    app.state.keyboxes.invalidate(tenant_id).await;

    // The newly generated signing keys are RS256; the legacy PS512 key only verifies now.
    let database = app.database().await;
    let keys = KeyBoxes::get_tenant_keys(tenant_id, &database)
        .await
        .expect("read tenant keys");
    let active = keys
        .iter()
        .filter(|key| key.status == KeyStatus::Active)
        .max_by_key(|key| key.activated_at)
        .expect("an active key exists after rotation");
    assert_eq!(
        active.key_alg,
        KeyAlg::Rs256,
        "the newly generated active key is RS256"
    );
    assert!(
        keys.iter()
            .filter(|key| matches!(key.status, KeyStatus::Active | KeyStatus::Pending))
            .all(|key| key.key_alg == KeyAlg::Rs256),
        "every signing-capable key is RS256 after the rotation, got {keys:?}"
    );
    assert!(
        keys.iter()
            .any(|key| key.status == KeyStatus::Retired && key.key_alg == KeyAlg::Ps512),
        "the legacy PS512 key remains as a retired verify-only key"
    );

    // Discovery advertisement derives from the live keys and has converged.
    assert_eq!(
        advertised_algs(&app, &fixture.tenant_sqid).await,
        ["RS256"],
        "post-rotation discovery must advertise the RS256 signing keys"
    );

    // A token issued after the rotation is signed by the new RS256 active key: the full
    // login → token exchange drive succeeds and the ID token header carries RS256.
    let (_, token_body) = drive_user_token_response(
        &app,
        &fixture,
        "key-transition@example.com",
        "key-transition-state",
    )
    .await;
    let id_token = token_body["id_token"]
        .as_str()
        .expect("token response should carry an id_token");
    let header = oceaniam_auth::decode_header(id_token).expect("id_token header should decode");
    assert_eq!(
        serde_json::to_value(header.alg).expect("alg serializes"),
        "RS256",
        "an ID token issued after the rotation is signed with RS256"
    );
}

/// A keybox whose keys are all retired advertises RS256 rather than a spec-invalid empty
/// list: the next rotation converges to the default algorithm, so it is the truthful outcome.
// NOTE: AI-generated test
#[tokio::test]
async fn all_retired_keybox_advertises_the_default_alg() {
    let app = spawn_preview().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_sqid = tenant["id"].as_str().unwrap().to_string();
    let tenant_id = sqid_to_uuid(&tenant_sqid);

    let database = app.database().await;
    KeyBoxes::update_many()
        .filter(Column::TenantId.eq(tenant_id))
        .set(key_boxes::ActiveModel {
            status: Set(KeyStatus::Retired),
            ..Default::default()
        })
        .exec(&database)
        .await
        .expect("retire every tenant key");
    app.state.keyboxes.invalidate(tenant_id).await;

    assert_eq!(
        advertised_algs(&app, &tenant_sqid).await,
        ["RS256"],
        "with no signing-capable keys the advertisement falls back to the default algorithm"
    );
}
