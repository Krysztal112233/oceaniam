use std::sync::Arc;

use chrono::Duration;
use oceaniam_common::crypto::MasterKey;
use oceaniam_database::{
    helper::{
        oidc_authorization_codes::{IssueAuthorizationCodeInput, OidcAuthorizationCodesHelper},
        oidc_authorization_transactions::{
            CreateAuthorizationTransactionInput, OidcAuthorizationTransactionsHelper,
        },
        oidc_clients::OidcClientsHelper,
    },
    model::{
        oidc_clients,
        prelude::{OidcAuthorizationCodes, OidcAuthorizationTransactions},
    },
};
use oceaniam_worker::{WorkerContext, collect_workers};
use sea_orm::{ConnectionTrait, DatabaseBackend, EntityTrait, Statement};
use uuid::Uuid;

use crate::support::{TestApp, spawn_app_with_isolated_schema};

const CODE_TTL: Duration = Duration::seconds(60);

/// Deterministic non-zero test KEK — not secret, tests only. The same fixed value as
/// `tests/support/mod.rs::TEST_MASTER_KEY_HEX`, kept private there.
const TEST_MASTER_KEY_HEX: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

struct SeededPair {
    transaction_id: Uuid,
    code_id: Uuid,
}

async fn seed_pair(app: &TestApp, label: &str, code_digest: [u8; 32]) -> SeededPair {
    let database = app.database().await;
    let (tenant_id, application_id) = app.seed_tenant_and_application().await;
    let oidc_client_id = Uuid::now_v7();
    oidc_clients::Entity::create_client(
        oidc_client_id,
        application_id,
        format!("cleanup-client-{label}"),
        format!("Cleanup client {label}"),
        &database,
    )
    .await
    .expect("create OIDC client owner");

    let transaction = OidcAuthorizationTransactions::create_authorization_transaction(
        CreateAuthorizationTransactionInput {
            tenant_id,
            application_id,
            oidc_client_id,
            issuer: format!("https://issuer.example/oidc/{label}"),
            redirect_uri: format!("https://client.example/{label}/callback"),
            state: format!("state-{label}"),
            nonce: None,
            code_challenge: "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM".to_owned(),
            browser_binding_digest: [0x11; 32],
            csrf_digest: [0x22; 32],
        },
        &database,
    )
    .await
    .expect("create authorization transaction");

    let code = OidcAuthorizationCodes::issue_authorization_code(
        IssueAuthorizationCodeInput {
            transaction_id: transaction.id,
            code_digest,
            redirect_uri: "https://client.example/callback".to_owned(),
            subject_id: Uuid::now_v7(),
        },
        CODE_TTL,
        &database,
    )
    .await
    .expect("issue authorization code");

    SeededPair {
        transaction_id: transaction.id,
        code_id: code.id,
    }
}

/// Rewrites `expires_at` with a database-clock SQL expression (tests legitimately seed state
/// with raw statements; the assertion targets keep the database clock as time authority).
async fn set_expires_at(app: &TestApp, table: &str, id: Uuid, clock_expression: &str) {
    let database = app.database().await;
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            format!("UPDATE {table} SET expires_at = {clock_expression} WHERE id = $1"),
            [id.into()],
        ))
        .await
        .expect("rewrite expires_at");
}

async fn transaction_exists(app: &TestApp, transaction_id: Uuid) -> bool {
    let database = app.database().await;
    OidcAuthorizationTransactions::find_by_id(transaction_id)
        .one(&database)
        .await
        .expect("read transaction")
        .is_some()
}

async fn code_exists(app: &TestApp, code_id: Uuid) -> bool {
    let database = app.database().await;
    OidcAuthorizationCodes::find_by_id(code_id)
        .one(&database)
        .await
        .expect("read code")
        .is_some()
}

async fn run_cleanup_once(app: &TestApp) {
    let database = app.database().await;
    let context = WorkerContext {
        database,
        master_key: Arc::new(MasterKey::from_hex(TEST_MASTER_KEY_HEX).expect("test master key")),
    };
    let workers = collect_workers();
    let worker = workers
        .get("oidc_expired_cleanup")
        .expect("oidc_expired_cleanup worker must be registered");
    worker.run(&context).await.expect("cleanup run succeeds");
}

// NOTE: AI-generated test
#[tokio::test]
async fn cleanup_deletes_aged_pairs_but_keeps_fresh_and_edge_window_rows() {
    let app = spawn_app_with_isolated_schema().await;

    // Aged pair: both rows expired more than the 24-hour forensics grace ago.
    let aged = seed_pair(&app, "aged", [0xA0; 32]).await;
    set_expires_at(
        &app,
        "oidc_authorization_transactions",
        aged.transaction_id,
        "clock_timestamp() - interval '25 hours'",
    )
    .await;
    set_expires_at(
        &app,
        "oidc_authorization_codes",
        aged.code_id,
        "clock_timestamp() - interval '25 hours'",
    )
    .await;

    // Fresh pair: defaults leave both rows well inside their TTLs.
    let fresh = seed_pair(&app, "fresh", [0xF0; 32]).await;

    // Edge window: the transaction's grace elapsed, but its code (60s TTL) outlives it by up
    // to a minute, so the transaction must survive this run.
    let edge = seed_pair(&app, "edge", [0xE0; 32]).await;
    set_expires_at(
        &app,
        "oidc_authorization_transactions",
        edge.transaction_id,
        "clock_timestamp() - interval '25 hours'",
    )
    .await;
    set_expires_at(
        &app,
        "oidc_authorization_codes",
        edge.code_id,
        "clock_timestamp() + interval '30 seconds'",
    )
    .await;

    run_cleanup_once(&app).await;

    assert!(
        !code_exists(&app, aged.code_id).await,
        "aged code must be deleted"
    );
    assert!(
        !transaction_exists(&app, aged.transaction_id).await,
        "aged transaction must be deleted"
    );
    assert!(
        code_exists(&app, fresh.code_id).await,
        "fresh code must survive"
    );
    assert!(
        transaction_exists(&app, fresh.transaction_id).await,
        "fresh transaction must survive"
    );
    assert!(
        code_exists(&app, edge.code_id).await,
        "edge-window code must survive"
    );
    assert!(
        transaction_exists(&app, edge.transaction_id).await,
        "transaction with a remaining code must survive"
    );

    // Once the edge code's grace elapses as well, the next run reclaims the transaction.
    set_expires_at(
        &app,
        "oidc_authorization_codes",
        edge.code_id,
        "clock_timestamp() - interval '25 hours'",
    )
    .await;
    run_cleanup_once(&app).await;

    assert!(
        !code_exists(&app, edge.code_id).await,
        "aged edge code must be deleted"
    );
    assert!(
        !transaction_exists(&app, edge.transaction_id).await,
        "transaction with no remaining code must be deleted on the next run"
    );
    assert!(
        code_exists(&app, fresh.code_id).await,
        "fresh code must survive the second run"
    );
    assert!(
        transaction_exists(&app, fresh.transaction_id).await,
        "fresh transaction must survive the second run"
    );
}
