use std::{sync::Arc, time::Duration};

use oceaniam_database::{
    Error,
    config::application::ApplicationConfiguration,
    helper::{
        applications::ApplicationHelper,
        credentials::CredentialsHelper,
        oidc_authorization_entry::{
            AuthorizationEntryLockResult, AuthorizationEntryResolution,
            OidcAuthorizationEntryHelper,
        },
        oidc_authorization_transactions::{
            AuthenticateAuthorizationTransactionInput,
            AuthenticateChallengedAuthorizationTransactionInput,
            ChallengeAuthorizationTransactionInput, CreateAuthorizationTransactionInput,
            OidcAuthorizationTransactionsHelper,
        },
        oidc_clients::OidcClientsHelper,
        subjects::SubjectsHelper,
    },
    model::{
        applications, oidc_authorization_transactions, oidc_clients,
        prelude::{
            Applications, Credentials, OidcAuthorizationTransactions, OidcClients, Subjects,
        },
        sea_orm_active_enums::{
            OidcAuthorizationTransactionStatus, OidcPkceMethod, SubjectTypeEnum,
        },
    },
};
use oceaniam_oidc::{RedirectUriPolicy, validate_redirect_uri};
use sea_orm::{
    ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, EntityTrait, ModelTrait,
    Statement, TransactionTrait,
};
use tokio::{sync::Barrier, time::timeout};
use uuid::Uuid;

use crate::support::{TestApp, spawn_app_with_isolated_schema};

const RFC_7636_CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
const BROWSER_DIGEST: [u8; 32] = [0x11; 32];
const CSRF_DIGEST: [u8; 32] = [0x22; 32];

#[derive(Clone)]
struct Owner {
    tenant_id: Uuid,
    application_id: Uuid,
    oidc_client_id: Uuid,
    client_id: String,
}

async fn seed_owner(app: &TestApp, label: &str) -> Owner {
    let database = app.database().await;
    let (tenant_id, application_id) = app.seed_tenant_and_application().await;
    let oidc_client_id = Uuid::now_v7();
    let client_id = format!("authorization-transaction-{label}");
    oidc_clients::Entity::create_client(
        oidc_client_id,
        application_id,
        client_id.clone(),
        format!("Authorization transaction {label}"),
        &database,
    )
    .await
    .expect("create OIDC client owner");

    Owner {
        tenant_id,
        application_id,
        oidc_client_id,
        client_id,
    }
}

fn create_input(
    owner: &Owner,
    marker: &str,
    browser_binding_digest: [u8; 32],
    csrf_digest: [u8; 32],
) -> CreateAuthorizationTransactionInput {
    CreateAuthorizationTransactionInput {
        tenant_id: owner.tenant_id,
        application_id: owner.application_id,
        oidc_client_id: owner.oidc_client_id,
        issuer: format!("https://issuer.example/oidc/{marker}"),
        redirect_uri: format!("https://client.example/{marker}/callback?fixed=true"),
        state: format!("sensitive-state-{marker}"),
        nonce: Some(format!("sensitive-nonce-{marker}")),
        code_challenge: RFC_7636_CHALLENGE.to_owned(),
        browser_binding_digest,
        csrf_digest,
    }
}

async fn create_transaction(
    database: &DatabaseConnection,
    owner: &Owner,
    marker: &str,
) -> oidc_authorization_transactions::Model {
    OidcAuthorizationTransactions::create_authorization_transaction(
        create_input(owner, marker, BROWSER_DIGEST, CSRF_DIGEST),
        database,
    )
    .await
    .expect("create authorization transaction")
}

fn assert_unavailable(error: &Error) -> String {
    match error {
        Error::CustomMessage { code, msg, .. } => {
            assert_eq!(*code, 404);
            assert_eq!(msg, "OIDC authorization transaction is unavailable");
        }
        other => panic!("expected generic unavailable error, got {other}"),
    }
    error.to_string()
}

async fn assert_update_rejected(
    database: &DatabaseConnection,
    transaction_id: Uuid,
    sql: &str,
    expected_error_fragment: &str,
) {
    let error = database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            sql,
            [transaction_id.into()],
        ))
        .await
        .expect_err("invalid authorization transaction update must be rejected");
    assert!(
        error.to_string().contains(expected_error_fragment),
        "expected error containing {expected_error_fragment:?}, got: {error}"
    );
}

async fn wait_until_connection_is_lock_blocked(
    observer: &DatabaseConnection,
    application_name: &str,
) {
    timeout(Duration::from_secs(2), async {
        loop {
            let waiting = observer
                .query_one_raw(Statement::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    "SELECT wait_event_type = 'Lock' AS waiting \
                     FROM pg_stat_activity \
                     WHERE application_name = $1 AND state = 'active' \
                     ORDER BY query_start DESC LIMIT 1",
                    [application_name.to_owned().into()],
                ))
                .await
                .expect("observe lock-waiting connection")
                .is_some_and(|row| row.try_get::<bool>("", "waiting").unwrap_or(false));
            if waiting {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("database operation must wait for the authorization shared lock");
}

// NOTE: AI-generated test
#[tokio::test]
async fn authorization_transaction_schema_enforces_the_storage_contract() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let owner = seed_owner(&app, "schema-primary").await;
    let transaction = create_transaction(&database, &owner, "schema-primary").await;

    assert_eq!(transaction.id.get_version_num(), 7);
    assert_eq!(transaction.tenant_id, owner.tenant_id);
    assert_eq!(transaction.application_id, owner.application_id);
    assert_eq!(transaction.oidc_client_id, owner.oidc_client_id);
    assert_eq!(
        transaction.issuer,
        "https://issuer.example/oidc/schema-primary"
    );
    assert_eq!(
        transaction.redirect_uri,
        "https://client.example/schema-primary/callback?fixed=true"
    );
    assert_eq!(transaction.requested_scope, "openid");
    assert_eq!(transaction.state, "sensitive-state-schema-primary");
    assert_eq!(
        transaction.nonce.as_deref(),
        Some("sensitive-nonce-schema-primary")
    );
    assert_eq!(transaction.code_challenge, RFC_7636_CHALLENGE);
    assert_eq!(transaction.code_challenge_method, OidcPkceMethod::S256);
    assert_eq!(transaction.browser_binding_digest, BROWSER_DIGEST);
    assert_eq!(transaction.csrf_digest, CSRF_DIGEST);
    assert_eq!(
        transaction.status,
        OidcAuthorizationTransactionStatus::Pending
    );
    assert_eq!(transaction.revision, 0);
    assert_eq!(transaction.terminal_at, None);
    assert_eq!(
        transaction.expires_at - transaction.created_at,
        chrono::Duration::minutes(10),
        "database defaults must produce an exact ten-minute lifetime"
    );

    let related_application = transaction
        .find_related(applications::Entity)
        .one(&database)
        .await
        .expect("query transaction's related Application")
        .expect("transaction must resolve its Application owner");
    assert_eq!(related_application.id, owner.application_id);
    let related_client = transaction
        .find_related(oidc_clients::Entity)
        .one(&database)
        .await
        .expect("query transaction's related OIDC client")
        .expect("transaction must resolve its OIDC client owner");
    assert_eq!(related_client.id, owner.oidc_client_id);
    assert_eq!(related_client.client_id, owner.client_id);

    let application_transactions = related_application
        .find_related(OidcAuthorizationTransactions)
        .all(&database)
        .await
        .expect("query an Application's related authorization transactions");
    assert_eq!(application_transactions, vec![transaction.clone()]);
    let client_transactions = related_client
        .find_related(OidcAuthorizationTransactions)
        .all(&database)
        .await
        .expect("query an OIDC client's related authorization transactions");
    assert_eq!(client_transactions, vec![transaction.clone()]);

    let enum_labels = database
        .query_all_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT type.typname, value.enumlabel \
             FROM pg_type AS type \
             JOIN pg_enum AS value ON value.enumtypid = type.oid \
             JOIN pg_namespace AS namespace ON namespace.oid = type.typnamespace \
             WHERE namespace.nspname = current_schema() \
               AND type.typname IN ( \
                   'oidc_authorization_transaction_status', 'oidc_pkce_method' \
               ) \
             ORDER BY type.typname, value.enumsortorder"
                .to_owned(),
        ))
        .await
        .expect("query authorization transaction enums")
        .into_iter()
        .map(|row| {
            (
                row.try_get::<String>("", "typname").unwrap(),
                row.try_get::<String>("", "enumlabel").unwrap(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        enum_labels,
        vec![
            (
                "oidc_authorization_transaction_status".to_owned(),
                "pending".to_owned(),
            ),
            (
                "oidc_authorization_transaction_status".to_owned(),
                "cancelled".to_owned(),
            ),
            (
                "oidc_authorization_transaction_status".to_owned(),
                "authenticated".to_owned(),
            ),
            (
                "oidc_authorization_transaction_status".to_owned(),
                "awaiting_challenge".to_owned(),
            ),
            ("oidc_pkce_method".to_owned(), "s256".to_owned()),
        ]
    );

    let binding_columns = database
        .query_all_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT column_name \
             FROM information_schema.columns \
             WHERE table_schema = current_schema() \
               AND table_name = 'oidc_authorization_transactions' \
               AND (column_name LIKE '%browser%' OR column_name LIKE '%csrf%') \
             ORDER BY column_name"
                .to_owned(),
        ))
        .await
        .expect("query browser and CSRF storage columns")
        .into_iter()
        .map(|row| row.try_get::<String>("", "column_name").unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        binding_columns,
        vec![
            "browser_binding_digest".to_owned(),
            "csrf_digest".to_owned(),
        ],
        "only digests may be persisted for browser binding and CSRF material"
    );

    let cascading_owner_foreign_keys = database
        .query_all_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT fk.conname, referenced.relname AS referenced_table \
             FROM pg_constraint AS fk \
             JOIN pg_class AS referenced ON referenced.oid = fk.confrelid \
             WHERE fk.conrelid = 'oidc_authorization_transactions'::regclass \
               AND fk.contype = 'f' \
               AND fk.confdeltype = 'c' \
             ORDER BY fk.conname"
                .to_owned(),
        ))
        .await
        .expect("query authorization transaction owner foreign keys")
        .into_iter()
        .map(|row| {
            (
                row.try_get::<String>("", "conname").unwrap(),
                row.try_get::<String>("", "referenced_table").unwrap(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        cascading_owner_foreign_keys,
        vec![
            (
                "fk_oidc_auth_tx_application_owner".to_owned(),
                "applications".to_owned(),
            ),
            (
                "fk_oidc_auth_tx_client_owner".to_owned(),
                "oidc_clients".to_owned(),
            ),
            ("fk_oidc_auth_tx_subject".to_owned(), "subjects".to_owned(),),
            ("fk_oidc_auth_tx_tenant".to_owned(), "tenants".to_owned(),),
        ],
        "only immutable owners, never redirect rows, are cascading FK targets"
    );

    let invalid_updates = [
        (
            "UPDATE oidc_authorization_transactions SET status = 'completed' WHERE id = $1",
            "oidc_authorization_transaction_status",
        ),
        (
            "UPDATE oidc_authorization_transactions SET code_challenge_method = 'plain' \
             WHERE id = $1",
            "oidc_pkce_method",
        ),
    ];
    for (sql, expected_error_fragment) in invalid_updates {
        assert_update_rejected(&database, transaction.id, sql, expected_error_fragment).await;
    }

    let second_owner = seed_owner(&app, "schema-second-owner").await;
    let wrong_tenant_marker = "owner-mismatch-sensitive-state";
    let mut wrong_tenant = create_input(&owner, wrong_tenant_marker, [0x33; 32], [0x44; 32]);
    wrong_tenant.tenant_id = second_owner.tenant_id;
    let error =
        OidcAuthorizationTransactions::create_authorization_transaction(wrong_tenant, &database)
            .await
            .expect_err("tenant/Application ownership mismatch must fail in the database");
    match &error {
        Error::CustomMessage { code, msg, .. } => {
            assert_eq!(*code, 500);
            assert_eq!(
                msg,
                "OIDC authorization transaction storage operation failed"
            );
        }
        other => panic!("expected sanitized storage error, got {other}"),
    }
    assert!(!error.to_string().contains(wrong_tenant_marker));

    let wrong_client_marker = "client-mismatch-sensitive-state";
    let mut wrong_client = create_input(&owner, wrong_client_marker, [0x55; 32], [0x66; 32]);
    wrong_client.oidc_client_id = second_owner.oidc_client_id;
    let error =
        OidcAuthorizationTransactions::create_authorization_transaction(wrong_client, &database)
            .await
            .expect_err("Application/client ownership mismatch must fail in the database");
    assert!(!error.to_string().contains(wrong_client_marker));
}

// NOTE: AI-generated test
#[tokio::test]
async fn owner_deletion_cascades_and_redirect_registration_is_not_a_foreign_key_target() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;

    let client_owner = seed_owner(&app, "delete-client").await;
    let client_transaction =
        create_transaction(&database, &client_owner, "unregistered-redirect").await;
    oidc_clients::Entity::delete_by_id(client_owner.oidc_client_id)
        .exec(&database)
        .await
        .expect("delete OIDC client");
    assert!(
        OidcAuthorizationTransactions::find_by_id(client_transaction.id)
            .one(&database)
            .await
            .expect("query client-owned transaction after deletion")
            .is_none()
    );
    assert_unavailable(
        &OidcAuthorizationTransactions::get_active_authorization_transaction(
            client_transaction.id,
            BROWSER_DIGEST,
            &database,
        )
        .await
        .expect_err("deleted client must make its transaction unavailable"),
    );

    let application_owner = seed_owner(&app, "delete-application").await;
    let application_transaction =
        create_transaction(&database, &application_owner, "delete-application").await;
    applications::Entity::delete_by_id(application_owner.application_id)
        .exec(&database)
        .await
        .expect("delete owning Application");
    assert!(
        OidcAuthorizationTransactions::find_by_id(application_transaction.id)
            .one(&database)
            .await
            .expect("query Application-owned transaction after deletion")
            .is_none()
    );

    let tenant_owner = seed_owner(&app, "delete-tenant").await;
    let tenant_transaction = create_transaction(&database, &tenant_owner, "delete-tenant").await;
    applications::Entity::delete_by_id(tenant_owner.application_id)
        .exec(&database)
        .await
        .expect("delete tenant's Application before its NO ACTION tenant relation");
    oceaniam_database::model::tenants::Entity::delete_by_id(tenant_owner.tenant_id)
        .exec(&database)
        .await
        .expect("delete owning tenant");
    assert!(
        OidcAuthorizationTransactions::find_by_id(tenant_transaction.id)
            .one(&database)
            .await
            .expect("query tenant-owned transaction after deletion")
            .is_none()
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn active_read_and_atomic_cancel_are_bound_expiring_and_revision_conditional() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let owner = seed_owner(&app, "lifecycle").await;
    let original = create_transaction(&database, &owner, "lifecycle").await;

    let active = OidcAuthorizationTransactions::get_active_authorization_transaction(
        original.id,
        BROWSER_DIGEST,
        &database,
    )
    .await
    .expect("correct browser binding should read a pending transaction");
    assert_eq!(active, original);

    let wrong_digest_error = assert_unavailable(
        &OidcAuthorizationTransactions::get_active_authorization_transaction(
            original.id,
            [0x99; 32],
            &database,
        )
        .await
        .expect_err("wrong browser binding must be indistinguishable from absence"),
    );

    let wrong_csrf_error = assert_unavailable(
        &OidcAuthorizationTransactions::cancel_authorization_transaction(
            original.id,
            BROWSER_DIGEST,
            [0x88; 32],
            original.revision,
            &database,
        )
        .await
        .expect_err("wrong CSRF binding must not cancel"),
    );
    assert_eq!(wrong_csrf_error, wrong_digest_error);

    let stale_revision_error = assert_unavailable(
        &OidcAuthorizationTransactions::cancel_authorization_transaction(
            original.id,
            BROWSER_DIGEST,
            CSRF_DIGEST,
            original.revision + 1,
            &database,
        )
        .await
        .expect_err("stale revision must not cancel"),
    );
    assert_eq!(stale_revision_error, wrong_digest_error);

    let still_pending = OidcAuthorizationTransactions::find_by_id(original.id)
        .one(&database)
        .await
        .expect("read transaction after rejected cancellation")
        .expect("transaction should remain stored");
    assert_eq!(
        still_pending.status,
        OidcAuthorizationTransactionStatus::Pending
    );
    assert_eq!(still_pending.revision, original.revision);
    assert_eq!(still_pending.terminal_at, None);

    let cancelled = OidcAuthorizationTransactions::cancel_authorization_transaction(
        original.id,
        BROWSER_DIGEST,
        CSRF_DIGEST,
        original.revision,
        &database,
    )
    .await
    .expect("matching revision and bindings should cancel");
    assert_eq!(
        cancelled.status,
        OidcAuthorizationTransactionStatus::Cancelled
    );
    assert_eq!(cancelled.revision, original.revision + 1);
    let terminal_at = cancelled
        .terminal_at
        .expect("cancelled transaction must have a terminal timestamp");
    assert!(terminal_at >= cancelled.created_at);
    assert!(terminal_at < cancelled.expires_at);
    assert_eq!(cancelled.created_at, original.created_at);
    assert_eq!(cancelled.expires_at, original.expires_at);
    assert_eq!(cancelled.tenant_id, original.tenant_id);
    assert_eq!(cancelled.application_id, original.application_id);
    assert_eq!(cancelled.oidc_client_id, original.oidc_client_id);
    assert_eq!(cancelled.issuer, original.issuer);
    assert_eq!(cancelled.redirect_uri, original.redirect_uri);
    assert_eq!(cancelled.requested_scope, original.requested_scope);
    assert_eq!(cancelled.state, original.state);
    assert_eq!(cancelled.nonce, original.nonce);
    assert_eq!(cancelled.code_challenge, original.code_challenge);
    assert_eq!(
        cancelled.code_challenge_method,
        original.code_challenge_method
    );
    assert_eq!(
        cancelled.browser_binding_digest,
        original.browser_binding_digest
    );
    assert_eq!(cancelled.csrf_digest, original.csrf_digest);

    let cancelled_error = assert_unavailable(
        &OidcAuthorizationTransactions::get_active_authorization_transaction(
            original.id,
            BROWSER_DIGEST,
            &database,
        )
        .await
        .expect_err("cancelled transaction must not be active"),
    );
    assert_eq!(cancelled_error, wrong_digest_error);
    assert_unavailable(
        &OidcAuthorizationTransactions::cancel_authorization_transaction(
            original.id,
            BROWSER_DIGEST,
            CSRF_DIGEST,
            cancelled.revision,
            &database,
        )
        .await
        .expect_err("cancel is terminal and cannot be repeated"),
    );

    let expiring = create_transaction(&database, &owner, "expired").await;
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "UPDATE oidc_authorization_transactions \
             SET created_at = created_at - interval '11 minutes', \
                 expires_at = expires_at - interval '11 minutes' \
             WHERE id = $1",
            [expiring.id.into()],
        ))
        .await
        .expect("expire transaction while preserving its fixed lifetime");

    let expired_read_error = assert_unavailable(
        &OidcAuthorizationTransactions::get_active_authorization_transaction(
            expiring.id,
            BROWSER_DIGEST,
            &database,
        )
        .await
        .expect_err("database-expired transaction must not be active"),
    );
    assert_eq!(expired_read_error, wrong_digest_error);
    let expired_cancel_error = assert_unavailable(
        &OidcAuthorizationTransactions::cancel_authorization_transaction(
            expiring.id,
            BROWSER_DIGEST,
            CSRF_DIGEST,
            expiring.revision,
            &database,
        )
        .await
        .expect_err("database-expired transaction must not cancel"),
    );
    assert_eq!(expired_cancel_error, wrong_digest_error);

    let expired = OidcAuthorizationTransactions::find_by_id(expiring.id)
        .one(&database)
        .await
        .expect("read expired transaction directly")
        .expect("expiry is a predicate and must not delete or restatus the row");
    assert_eq!(expired.status, OidcAuthorizationTransactionStatus::Pending);
    assert_eq!(expired.revision, 0);
    assert_eq!(expired.terminal_at, None);
    assert_eq!(
        expired.expires_at - expired.created_at,
        chrono::Duration::minutes(10)
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn cancellation_rechecks_expiry_after_waiting_for_the_row_lock() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let owner = seed_owner(&app, "expiry-after-lock").await;
    let transaction = create_transaction(&database, &owner, "expiry-after-lock").await;

    let blocker = database
        .begin()
        .await
        .expect("begin transaction that holds the authorization row lock");
    blocker
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT id FROM oidc_authorization_transactions WHERE id = $1 FOR UPDATE",
            [transaction.id.into()],
        ))
        .await
        .expect("lock authorization transaction")
        .expect("authorization transaction should exist");

    let application_name = format!("oidc_auth_tx_expiry_{}", Uuid::now_v7().simple());
    let cancellation_database = Database::connect(format!(
        "{}&application_name={application_name}",
        app.dsn_with_schema()
    ))
    .await
    .expect("connect dedicated cancellation database");
    let transaction_id = transaction.id;
    let transaction_revision = transaction.revision;
    let cancellation = tokio::spawn(async move {
        OidcAuthorizationTransactions::cancel_authorization_transaction(
            transaction_id,
            BROWSER_DIGEST,
            CSRF_DIGEST,
            transaction_revision,
            &cancellation_database,
        )
        .await
    });

    timeout(Duration::from_secs(2), async {
        loop {
            let waiting = database
                .query_one_raw(Statement::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    "SELECT wait_event_type = 'Lock' AS waiting \
                     FROM pg_stat_activity \
                     WHERE application_name = $1 AND state = 'active' \
                     ORDER BY query_start DESC LIMIT 1",
                    [application_name.clone().into()],
                ))
                .await
                .expect("observe cancellation connection")
                .is_some_and(|row| row.try_get::<bool>("", "waiting").unwrap_or(false));
            if waiting {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("cancellation must wait for the authorization row lock");

    blocker
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "WITH sampled AS (SELECT clock_timestamp() AS now) \
             UPDATE oidc_authorization_transactions \
             SET created_at = sampled.now - interval '9 minutes 59.8 seconds', \
                 expires_at = sampled.now + interval '0.2 seconds' \
             FROM sampled WHERE id = $1",
            [transaction.id.into()],
        ))
        .await
        .expect("shorten the lifetime while preserving the exact ten-minute interval");
    tokio::time::sleep(Duration::from_millis(300)).await;
    blocker
        .commit()
        .await
        .expect("release the row only after the transaction expires");

    let error = cancellation
        .await
        .expect("cancellation task must not panic")
        .expect_err("time spent waiting for a row lock must not revive an expired transaction");
    assert_unavailable(&error);
}

// NOTE: AI-generated test
#[tokio::test]
async fn concurrent_cancellations_have_exactly_one_winner() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let owner = seed_owner(&app, "concurrent-cancel").await;
    let transaction = create_transaction(&database, &owner, "concurrent-cancel").await;
    let barrier = Arc::new(Barrier::new(3));

    let cancel = |database: DatabaseConnection, barrier: Arc<Barrier>| {
        tokio::spawn(async move {
            barrier.wait().await;
            OidcAuthorizationTransactions::cancel_authorization_transaction(
                transaction.id,
                BROWSER_DIGEST,
                CSRF_DIGEST,
                transaction.revision,
                &database,
            )
            .await
        })
    };
    let first = cancel(database.clone(), Arc::clone(&barrier));
    let second = cancel(database.clone(), Arc::clone(&barrier));
    barrier.wait().await;

    let first = first.await.expect("first cancellation task must not panic");
    let second = second
        .await
        .expect("second cancellation task must not panic");
    assert_eq!(
        usize::from(first.is_ok()) + usize::from(second.is_ok()),
        1,
        "the conditional UPDATE must return exactly one concurrent winner"
    );

    let winner = first.or(second).expect("one cancellation must win");
    assert_eq!(winner.status, OidcAuthorizationTransactionStatus::Cancelled);
    assert_eq!(winner.revision, transaction.revision + 1);
    assert!(winner.terminal_at.is_some());

    let stored = OidcAuthorizationTransactions::find_by_id(transaction.id)
        .one(&database)
        .await
        .expect("read concurrently cancelled transaction")
        .expect("cancelled transaction remains stored");
    assert_eq!(stored.status, OidcAuthorizationTransactionStatus::Cancelled);
    assert_eq!(stored.revision, 1);
    assert!(stored.terminal_at.is_some());
    assert_eq!(stored.created_at, transaction.created_at);
    assert_eq!(stored.expires_at, transaction.expires_at);
}

// NOTE: AI-generated test
#[tokio::test]
async fn authorization_entry_locks_serialize_redirect_patch_before_snapshot_creation() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let owner = seed_owner(&app, "entry-redirect-lock").await;
    let original_redirect = "https://client.example/original-callback";
    let replacement_redirect = "https://client.example/replacement-callback";
    OidcClients::create_redirect_uris(
        owner.oidc_client_id,
        vec![original_redirect.to_owned()],
        &database,
    )
    .await
    .expect("register original redirect");

    let authorization = database
        .begin()
        .await
        .expect("begin authorization transaction");
    let candidate = match OidcClients::resolve_authorization_entry_candidate(
        owner.tenant_id,
        &owner.client_id,
        &authorization,
    )
    .await
    .expect("resolve authorization candidate")
    {
        AuthorizationEntryResolution::Candidate(candidate) => candidate,
        AuthorizationEntryResolution::TenantUnavailable
        | AuthorizationEntryResolution::ClientUnavailable => {
            panic!("seeded authorization owner must resolve")
        }
    };
    let locked = match OidcClients::lock_authorization_entry_registration(
        owner.tenant_id,
        candidate,
        &owner.client_id,
        original_redirect,
        &authorization,
    )
    .await
    .expect("lock live authorization registration")
    {
        AuthorizationEntryLockResult::Locked(locked) => locked,
        AuthorizationEntryLockResult::TenantUnavailable
        | AuthorizationEntryLockResult::RelationshipUnavailable => {
            panic!("seeded authorization registration must lock")
        }
    };

    let application_name = format!("oidc_entry_patch_{}", Uuid::now_v7().simple());
    let patch_database = Database::connect(format!(
        "{}&application_name={application_name}",
        app.dsn_with_schema()
    ))
    .await
    .expect("connect dedicated redirect patch database");
    let patch_application_id = owner.application_id;
    let patch_client_id = owner.client_id.clone();
    let patch = tokio::spawn(async move {
        let patch_transaction = patch_database.begin().await.expect("begin redirect patch");
        let client = OidcClients::get_client_for_update(
            patch_application_id,
            &patch_client_id,
            &patch_transaction,
        )
        .await
        .expect("lock client for redirect patch");
        OidcClients::replace_redirect_uris(
            client.id,
            vec![replacement_redirect.to_owned()],
            &patch_transaction,
        )
        .await
        .expect("replace redirect registration");
        patch_transaction
            .commit()
            .await
            .expect("commit redirect patch");
    });

    wait_until_connection_is_lock_blocked(&database, &application_name).await;
    assert!(
        !patch.is_finished(),
        "redirect patch must remain blocked while authorization holds shared locks"
    );

    let mut input = create_input(&owner, "entry-redirect-lock", BROWSER_DIGEST, CSRF_DIGEST);
    input.application_id = locked.application_id;
    input.oidc_client_id = locked.oidc_client_id;
    input.redirect_uri = original_redirect.to_owned();
    let snapshot =
        OidcAuthorizationTransactions::create_authorization_transaction(input, &authorization)
            .await
            .expect("insert snapshot under registration locks");
    authorization
        .commit()
        .await
        .expect("commit authorization snapshot");
    patch.await.expect("redirect patch task must complete");

    let stored = OidcAuthorizationTransactions::find_by_id(snapshot.id)
        .one(&database)
        .await
        .expect("read snapshot after redirect patch")
        .expect("snapshot remains independently stored");
    assert_eq!(stored.redirect_uri, original_redirect);

    let recheck = database.begin().await.expect("begin registration recheck");
    let candidate = match OidcClients::resolve_authorization_entry_candidate(
        owner.tenant_id,
        &owner.client_id,
        &recheck,
    )
    .await
    .expect("resolve candidate after patch")
    {
        AuthorizationEntryResolution::Candidate(candidate) => candidate,
        AuthorizationEntryResolution::TenantUnavailable
        | AuthorizationEntryResolution::ClientUnavailable => {
            panic!("client should remain after redirect patch")
        }
    };
    assert!(matches!(
        OidcClients::lock_authorization_entry_registration(
            owner.tenant_id,
            candidate,
            &owner.client_id,
            original_redirect,
            &recheck,
        )
        .await
        .expect("recheck old redirect"),
        AuthorizationEntryLockResult::RelationshipUnavailable
    ));
    recheck.rollback().await.expect("roll back recheck");
}

// NOTE: AI-generated test
#[tokio::test]
async fn authorization_entry_locks_serialize_client_delete_and_owner_cascade() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let owner = seed_owner(&app, "entry-client-delete-lock").await;
    let redirect = "https://client.example/delete-callback";
    OidcClients::create_redirect_uris(owner.oidc_client_id, vec![redirect.to_owned()], &database)
        .await
        .expect("register redirect before delete race");

    let authorization = database
        .begin()
        .await
        .expect("begin authorization transaction");
    let candidate = match OidcClients::resolve_authorization_entry_candidate(
        owner.tenant_id,
        &owner.client_id,
        &authorization,
    )
    .await
    .expect("resolve authorization candidate")
    {
        AuthorizationEntryResolution::Candidate(candidate) => candidate,
        AuthorizationEntryResolution::TenantUnavailable
        | AuthorizationEntryResolution::ClientUnavailable => {
            panic!("seeded authorization owner must resolve")
        }
    };
    let locked = match OidcClients::lock_authorization_entry_registration(
        owner.tenant_id,
        candidate,
        &owner.client_id,
        redirect,
        &authorization,
    )
    .await
    .expect("lock live authorization registration")
    {
        AuthorizationEntryLockResult::Locked(locked) => locked,
        AuthorizationEntryLockResult::TenantUnavailable
        | AuthorizationEntryLockResult::RelationshipUnavailable => {
            panic!("seeded authorization registration must lock")
        }
    };

    let application_name = format!("oidc_entry_delete_{}", Uuid::now_v7().simple());
    let delete_database = Database::connect(format!(
        "{}&application_name={application_name}",
        app.dsn_with_schema()
    ))
    .await
    .expect("connect dedicated client delete database");
    let delete_application_id = owner.application_id;
    let delete_client_id = owner.client_id.clone();
    let deletion = tokio::spawn(async move {
        let delete_transaction = delete_database.begin().await.expect("begin client delete");
        let client = OidcClients::get_client_for_update(
            delete_application_id,
            &delete_client_id,
            &delete_transaction,
        )
        .await
        .expect("lock client for deletion");
        OidcClients::delete_locked_client(client.id, &delete_transaction)
            .await
            .expect("delete locked client");
        delete_transaction
            .commit()
            .await
            .expect("commit client deletion");
    });

    wait_until_connection_is_lock_blocked(&database, &application_name).await;
    assert!(
        !deletion.is_finished(),
        "client deletion must remain blocked while authorization holds shared locks"
    );

    let mut input = create_input(
        &owner,
        "entry-client-delete-lock",
        BROWSER_DIGEST,
        CSRF_DIGEST,
    );
    input.application_id = locked.application_id;
    input.oidc_client_id = locked.oidc_client_id;
    input.redirect_uri = redirect.to_owned();
    let snapshot =
        OidcAuthorizationTransactions::create_authorization_transaction(input, &authorization)
            .await
            .expect("insert snapshot before serialized client deletion");
    authorization
        .commit()
        .await
        .expect("commit authorization snapshot");
    deletion.await.expect("client deletion task must complete");

    assert!(
        OidcAuthorizationTransactions::find_by_id(snapshot.id)
            .one(&database)
            .await
            .expect("query snapshot after client deletion")
            .is_none(),
        "the serialized owner delete must cascade the previously committed snapshot"
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn authorization_entry_locks_serialize_application_redirect_policy_changes() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let owner = seed_owner(&app, "entry-policy-lock").await;
    let redirect = "http://localhost:3000/callback";
    OidcClients::create_redirect_uris(owner.oidc_client_id, vec![redirect.to_owned()], &database)
        .await
        .expect("register loopback redirect");
    let mut allowed_configuration = ApplicationConfiguration::default();
    allowed_configuration
        .oidc
        .allow_insecure_loopback_redirect_uris = true;
    Applications::replace_configuration(owner.application_id, allowed_configuration, &database)
        .await
        .expect("enable loopback redirect policy");

    let authorization = database
        .begin()
        .await
        .expect("begin authorization transaction");
    let candidate = match OidcClients::resolve_authorization_entry_candidate(
        owner.tenant_id,
        &owner.client_id,
        &authorization,
    )
    .await
    .expect("resolve authorization candidate")
    {
        AuthorizationEntryResolution::Candidate(candidate) => candidate,
        AuthorizationEntryResolution::TenantUnavailable
        | AuthorizationEntryResolution::ClientUnavailable => {
            panic!("seeded authorization owner must resolve")
        }
    };
    let locked = match OidcClients::lock_authorization_entry_registration(
        owner.tenant_id,
        candidate,
        &owner.client_id,
        redirect,
        &authorization,
    )
    .await
    .expect("lock live loopback registration")
    {
        AuthorizationEntryLockResult::Locked(locked) => locked,
        AuthorizationEntryLockResult::TenantUnavailable
        | AuthorizationEntryLockResult::RelationshipUnavailable => {
            panic!("seeded loopback registration must lock")
        }
    };
    let locked_configuration: ApplicationConfiguration =
        serde_json::from_value(locked.application_configuration.clone())
            .expect("locked Application configuration should deserialize");
    assert!(
        locked_configuration
            .oidc
            .allow_insecure_loopback_redirect_uris
    );
    validate_redirect_uri(redirect, RedirectUriPolicy::web_with_insecure_loopback())
        .expect("locked policy should allow loopback redirect");

    let application_name = format!("oidc_entry_policy_{}", Uuid::now_v7().simple());
    let update_database = Database::connect(format!(
        "{}&application_name={application_name}",
        app.dsn_with_schema()
    ))
    .await
    .expect("connect dedicated policy update database");
    let update_application_id = owner.application_id;
    let update = tokio::spawn(async move {
        Applications::replace_configuration(
            update_application_id,
            ApplicationConfiguration::default(),
            &update_database,
        )
        .await
        .expect("disable loopback redirect policy");
    });

    wait_until_connection_is_lock_blocked(&database, &application_name).await;
    assert!(
        !update.is_finished(),
        "policy update must remain blocked while authorization holds the Application lock"
    );

    let mut input = create_input(&owner, "entry-policy-lock", BROWSER_DIGEST, CSRF_DIGEST);
    input.application_id = locked.application_id;
    input.oidc_client_id = locked.oidc_client_id;
    input.redirect_uri = redirect.to_owned();
    let snapshot =
        OidcAuthorizationTransactions::create_authorization_transaction(input, &authorization)
            .await
            .expect("insert snapshot under the locked allow policy");
    authorization
        .commit()
        .await
        .expect("commit authorization snapshot");
    update.await.expect("policy update task must complete");

    let stored = OidcAuthorizationTransactions::find_by_id(snapshot.id)
        .one(&database)
        .await
        .expect("query policy-race snapshot")
        .expect("snapshot committed before policy update");
    assert_eq!(stored.redirect_uri, redirect);
    let current = Applications::get_application(owner.application_id, &database)
        .await
        .expect("read current Application after policy update");
    let current: ApplicationConfiguration = serde_json::from_value(current.configuration)
        .expect("current Application configuration should deserialize");
    assert!(!current.oidc.allow_insecure_loopback_redirect_uris);
    assert_eq!(
        validate_redirect_uri(redirect, RedirectUriPolicy::web()),
        Err(oceaniam_oidc::RedirectUriError::HttpsRequired),
        "a later authorization attempt must reject the now-disabled loopback policy"
    );
}

const SUBJECT_ID: Uuid = Uuid::from_u128(0x018fc2ab_bd4e_7c2b_8f3a_2a1c4d5e6f70);

/// Every isolated schema enforces the subject FK, so a successful authentication needs a real
/// subject row for the hardcoded test subject. `subjects.id` references `credentials.id`, so
/// the credential row is created first (the same ordering `create_user` documents).
async fn seed_subject(database: &DatabaseConnection, owner: &Owner) {
    Credentials::upsert_credential(SUBJECT_ID, "test-phc".to_owned(), None, database)
        .await
        .expect("seed the authenticated subject's credential");
    Subjects::create_subjects(
        SUBJECT_ID,
        owner.application_id,
        SubjectTypeEnum::User,
        database,
    )
    .await
    .expect("seed the authenticated subject");
}

fn authenticate_input(
    transaction: &oidc_authorization_transactions::Model,
) -> AuthenticateAuthorizationTransactionInput {
    AuthenticateAuthorizationTransactionInput {
        id: transaction.id,
        browser_binding_digest: BROWSER_DIGEST,
        csrf_digest: CSRF_DIGEST,
        expected_revision: transaction.revision,
        subject_id: SUBJECT_ID,
    }
}

async fn authenticate_in_owned_transaction(
    database: &DatabaseConnection,
    input: AuthenticateAuthorizationTransactionInput,
) -> Result<oidc_authorization_transactions::Model, Error> {
    let transaction = database.begin().await.expect("begin login transaction");
    let result =
        OidcAuthorizationTransactions::authenticate_authorization_transaction(input, &transaction)
            .await;
    match result {
        Ok(model) => {
            transaction
                .commit()
                .await
                .expect("commit login transaction");
            Ok(model)
        }
        Err(error) => {
            transaction
                .rollback()
                .await
                .expect("roll back login transaction");
            Err(error)
        }
    }
}

// NOTE: AI-generated test
#[tokio::test]
async fn authorization_login_locks_serialize_redirect_patch_before_authentication() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let owner = seed_owner(&app, "login-redirect-lock").await;
    seed_subject(&database, &owner).await;
    let original_redirect = "https://client.example/login-original-callback";
    let replacement_redirect = "https://client.example/login-replacement-callback";
    OidcClients::create_redirect_uris(
        owner.oidc_client_id,
        vec![original_redirect.to_owned()],
        &database,
    )
    .await
    .expect("register original redirect");
    let mut input = create_input(&owner, "login-redirect-lock", BROWSER_DIGEST, CSRF_DIGEST);
    input.redirect_uri = original_redirect.to_owned();
    let transaction =
        OidcAuthorizationTransactions::create_authorization_transaction(input, &database)
            .await
            .expect("create pending transaction for the login lock race");

    let login = database.begin().await.expect("begin login transaction");
    let locked = match OidcClients::lock_authorization_login_registration(
        owner.tenant_id,
        owner.application_id,
        owner.oidc_client_id,
        original_redirect,
        &login,
    )
    .await
    .expect("lock live login registration")
    {
        AuthorizationEntryLockResult::Locked(locked) => locked,
        AuthorizationEntryLockResult::TenantUnavailable
        | AuthorizationEntryLockResult::RelationshipUnavailable => {
            panic!("seeded login registration must lock")
        }
    };
    assert_eq!(locked.application_id, owner.application_id);
    assert_eq!(locked.oidc_client_id, owner.oidc_client_id);
    let locked_configuration: ApplicationConfiguration =
        serde_json::from_value(locked.application_configuration.clone())
            .expect("locked Application configuration should deserialize");
    let locked_policy = if locked_configuration
        .oidc
        .allow_insecure_loopback_redirect_uris
    {
        RedirectUriPolicy::web_with_insecure_loopback()
    } else {
        RedirectUriPolicy::web()
    };
    validate_redirect_uri(original_redirect, locked_policy)
        .expect("locked registration should allow the snapshot redirect");

    let application_name = format!("oidc_login_patch_{}", Uuid::now_v7().simple());
    let patch_database = Database::connect(format!(
        "{}&application_name={application_name}",
        app.dsn_with_schema()
    ))
    .await
    .expect("connect dedicated redirect patch database");
    let patch_application_id = owner.application_id;
    let patch_client_id = owner.client_id.clone();
    let patch = tokio::spawn(async move {
        let patch_transaction = patch_database.begin().await.expect("begin redirect patch");
        let client = OidcClients::get_client_for_update(
            patch_application_id,
            &patch_client_id,
            &patch_transaction,
        )
        .await
        .expect("lock client for redirect patch");
        OidcClients::replace_redirect_uris(
            client.id,
            vec![replacement_redirect.to_owned()],
            &patch_transaction,
        )
        .await
        .expect("replace redirect registration");
        patch_transaction
            .commit()
            .await
            .expect("commit redirect patch");
    });

    wait_until_connection_is_lock_blocked(&database, &application_name).await;
    assert!(
        !patch.is_finished(),
        "redirect patch must remain blocked while the login holds the registration shared locks"
    );

    let authenticated = OidcAuthorizationTransactions::authenticate_authorization_transaction(
        authenticate_input(&transaction),
        &login,
    )
    .await
    .expect("authenticate under the login registration locks");
    assert_eq!(
        authenticated.status,
        OidcAuthorizationTransactionStatus::Authenticated
    );
    assert_eq!(authenticated.revision, transaction.revision + 1);
    assert_eq!(authenticated.subject_id, Some(SUBJECT_ID));
    assert!(authenticated.authenticated_at.is_some());
    login.commit().await.expect("commit the serialized login");
    patch.await.expect("redirect patch task must complete");

    let stored = OidcAuthorizationTransactions::find_by_id(transaction.id)
        .one(&database)
        .await
        .expect("read transaction after the serialized redirect patch")
        .expect("authentication must remain committed");
    assert_eq!(
        stored.status,
        OidcAuthorizationTransactionStatus::Authenticated
    );
    assert_eq!(stored.revision, 1);
    assert_eq!(stored.subject_id, Some(SUBJECT_ID));
    assert_eq!(stored.redirect_uri, original_redirect);

    let recheck = database.begin().await.expect("begin login recheck");
    assert!(
        matches!(
            OidcClients::lock_authorization_login_registration(
                owner.tenant_id,
                owner.application_id,
                owner.oidc_client_id,
                original_redirect,
                &recheck,
            )
            .await
            .expect("recheck patched-away redirect"),
            AuthorizationEntryLockResult::RelationshipUnavailable
        ),
        "a later login must reject the redirect the serialized patch removed"
    );
    assert!(
        matches!(
            OidcClients::lock_authorization_login_registration(
                owner.tenant_id,
                owner.application_id,
                owner.oidc_client_id,
                replacement_redirect,
                &recheck,
            )
            .await
            .expect("recheck replacement redirect"),
            AuthorizationEntryLockResult::Locked(_)
        ),
        "a later login must see the replacement redirect"
    );
    recheck.rollback().await.expect("roll back login recheck");
}

// NOTE: AI-generated test
#[tokio::test]
async fn authenticate_transitions_pending_to_authenticated_and_records_subject() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let owner = seed_owner(&app, "authenticate").await;
    seed_subject(&database, &owner).await;
    let original = create_transaction(&database, &owner, "authenticate").await;

    let login_read = OidcAuthorizationTransactions::get_login_authorization_transaction(
        original.id,
        owner.tenant_id,
        BROWSER_DIGEST,
        CSRF_DIGEST,
        &database,
    )
    .await
    .expect("fully bound pending transaction should be readable for login");
    assert_eq!(login_read, original);

    let authenticated = authenticate_in_owned_transaction(&database, authenticate_input(&original))
        .await
        .expect("matching bindings and revision should authenticate");
    assert_eq!(
        authenticated.status,
        OidcAuthorizationTransactionStatus::Authenticated
    );
    assert_eq!(authenticated.revision, original.revision + 1);
    assert_eq!(authenticated.subject_id, Some(SUBJECT_ID));
    let authenticated_at = authenticated
        .authenticated_at
        .expect("authenticated transaction must record the database clock");
    assert!(authenticated_at >= authenticated.created_at);
    assert!(authenticated_at < authenticated.expires_at);
    assert!(authenticated.terminal_at.is_none());
    assert_eq!(authenticated.created_at, original.created_at);
    assert_eq!(authenticated.expires_at, original.expires_at);
    assert_eq!(authenticated.tenant_id, original.tenant_id);
    assert_eq!(authenticated.application_id, original.application_id);
    assert_eq!(authenticated.oidc_client_id, original.oidc_client_id);
    assert_eq!(authenticated.issuer, original.issuer);
    assert_eq!(authenticated.redirect_uri, original.redirect_uri);
    assert_eq!(authenticated.state, original.state);
    assert_eq!(authenticated.nonce, original.nonce);
    assert_eq!(
        authenticated.browser_binding_digest,
        original.browser_binding_digest
    );
    assert_eq!(authenticated.csrf_digest, original.csrf_digest);

    let generic_error = assert_unavailable(
        &OidcAuthorizationTransactions::get_active_authorization_transaction(
            original.id,
            BROWSER_DIGEST,
            &database,
        )
        .await
        .expect_err("authenticated transaction must no longer be active"),
    );
    let login_read_error = assert_unavailable(
        &OidcAuthorizationTransactions::get_login_authorization_transaction(
            original.id,
            owner.tenant_id,
            BROWSER_DIGEST,
            CSRF_DIGEST,
            &database,
        )
        .await
        .expect_err("authenticated transaction must not be readable for login"),
    );
    assert_eq!(login_read_error, generic_error);
    let repeat_error = assert_unavailable(
        &authenticate_in_owned_transaction(&database, authenticate_input(&authenticated))
            .await
            .expect_err("authentication is terminal and cannot be repeated"),
    );
    assert_eq!(repeat_error, generic_error);

    let stored = OidcAuthorizationTransactions::find_by_id(original.id)
        .one(&database)
        .await
        .expect("read authenticated transaction directly")
        .expect("authentication must not delete the row");
    assert_eq!(stored, authenticated);
}

// NOTE: AI-generated test
#[tokio::test]
async fn authenticate_requires_fresh_pending_state_and_matching_bindings() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let owner = seed_owner(&app, "authenticate-reject").await;
    let other_owner = seed_owner(&app, "authenticate-reject-other").await;
    let original = create_transaction(&database, &owner, "authenticate-reject").await;

    let wrong_browser = assert_unavailable(
        &OidcAuthorizationTransactions::get_login_authorization_transaction(
            original.id,
            owner.tenant_id,
            [0x99; 32],
            CSRF_DIGEST,
            &database,
        )
        .await
        .expect_err("wrong browser binding must be rejected before login"),
    );
    let wrong_csrf = assert_unavailable(
        &OidcAuthorizationTransactions::get_login_authorization_transaction(
            original.id,
            owner.tenant_id,
            BROWSER_DIGEST,
            [0x88; 32],
            &database,
        )
        .await
        .expect_err("wrong CSRF binding must be rejected before login"),
    );
    assert_eq!(wrong_csrf, wrong_browser);
    let wrong_tenant = assert_unavailable(
        &OidcAuthorizationTransactions::get_login_authorization_transaction(
            original.id,
            other_owner.tenant_id,
            BROWSER_DIGEST,
            CSRF_DIGEST,
            &database,
        )
        .await
        .expect_err("a cross-tenant read must be indistinguishable from absence"),
    );
    assert_eq!(wrong_tenant, wrong_browser);

    let mut wrong_browser_input = authenticate_input(&original);
    wrong_browser_input.browser_binding_digest = [0x99; 32];
    let wrong_browser_error = assert_unavailable(
        &authenticate_in_owned_transaction(&database, wrong_browser_input)
            .await
            .expect_err("wrong browser binding must not authenticate"),
    );
    assert_eq!(wrong_browser_error, wrong_browser);

    let mut wrong_csrf_input = authenticate_input(&original);
    wrong_csrf_input.csrf_digest = [0x88; 32];
    let wrong_csrf_error = assert_unavailable(
        &authenticate_in_owned_transaction(&database, wrong_csrf_input)
            .await
            .expect_err("wrong CSRF binding must not authenticate"),
    );
    assert_eq!(wrong_csrf_error, wrong_browser);

    let mut stale_revision_input = authenticate_input(&original);
    stale_revision_input.expected_revision = original.revision + 1;
    let stale_revision_error = assert_unavailable(
        &authenticate_in_owned_transaction(&database, stale_revision_input)
            .await
            .expect_err("a stale revision must not authenticate"),
    );
    assert_eq!(stale_revision_error, wrong_browser);

    let still_pending = OidcAuthorizationTransactions::find_by_id(original.id)
        .one(&database)
        .await
        .expect("read transaction after rejected authentication attempts")
        .expect("transaction should remain stored");
    assert_eq!(
        still_pending.status,
        OidcAuthorizationTransactionStatus::Pending
    );
    assert_eq!(still_pending.revision, original.revision);
    assert_eq!(still_pending.subject_id, None);
    assert_eq!(still_pending.authenticated_at, None);

    let expiring = create_transaction(&database, &owner, "authenticate-expired").await;
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "UPDATE oidc_authorization_transactions \
             SET created_at = created_at - interval '11 minutes', \
                 expires_at = expires_at - interval '11 minutes' \
             WHERE id = $1",
            [expiring.id.into()],
        ))
        .await
        .expect("expire transaction while preserving its fixed lifetime");
    let expired_read_error = assert_unavailable(
        &OidcAuthorizationTransactions::get_login_authorization_transaction(
            expiring.id,
            owner.tenant_id,
            BROWSER_DIGEST,
            CSRF_DIGEST,
            &database,
        )
        .await
        .expect_err("database-expired transaction must not be readable for login"),
    );
    assert_eq!(expired_read_error, wrong_browser);
    let expired_authenticate_error = assert_unavailable(
        &authenticate_in_owned_transaction(&database, authenticate_input(&expiring))
            .await
            .expect_err("database-expired transaction must not authenticate"),
    );
    assert_eq!(expired_authenticate_error, wrong_browser);

    let cancelled = OidcAuthorizationTransactions::cancel_authorization_transaction(
        original.id,
        BROWSER_DIGEST,
        CSRF_DIGEST,
        original.revision,
        &database,
    )
    .await
    .expect("cancel the original transaction");
    let cancelled_read_error = assert_unavailable(
        &OidcAuthorizationTransactions::get_login_authorization_transaction(
            original.id,
            owner.tenant_id,
            BROWSER_DIGEST,
            CSRF_DIGEST,
            &database,
        )
        .await
        .expect_err("cancelled transaction must not be readable for login"),
    );
    assert_eq!(cancelled_read_error, wrong_browser);
    let cancelled_authenticate_error = assert_unavailable(
        &authenticate_in_owned_transaction(&database, authenticate_input(&cancelled))
            .await
            .expect_err("a non-pending start state must not authenticate"),
    );
    assert_eq!(cancelled_authenticate_error, wrong_browser);
}

// NOTE: AI-generated test
#[tokio::test]
async fn authenticate_rechecks_expiry_after_waiting_for_the_row_lock() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let owner = seed_owner(&app, "authenticate-expiry-lock").await;
    let transaction = create_transaction(&database, &owner, "authenticate-expiry-lock").await;

    let blocker = database
        .begin()
        .await
        .expect("begin transaction that holds the authorization row lock");
    blocker
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT id FROM oidc_authorization_transactions WHERE id = $1 FOR UPDATE",
            [transaction.id.into()],
        ))
        .await
        .expect("lock authorization transaction")
        .expect("authorization transaction should exist");

    let application_name = format!("oidc_auth_tx_auth_{}", Uuid::now_v7().simple());
    let authentication_database = Database::connect(format!(
        "{}&application_name={application_name}",
        app.dsn_with_schema()
    ))
    .await
    .expect("connect dedicated authentication database");
    let input = authenticate_input(&transaction);
    let authentication = tokio::spawn(async move {
        let transaction = authentication_database
            .begin()
            .await
            .expect("begin blocked login transaction");
        let result = OidcAuthorizationTransactions::authenticate_authorization_transaction(
            input,
            &transaction,
        )
        .await;
        transaction
            .rollback()
            .await
            .expect("roll back blocked login transaction");
        result
    });

    timeout(Duration::from_secs(2), async {
        loop {
            let waiting = database
                .query_one_raw(Statement::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    "SELECT wait_event_type = 'Lock' AS waiting \
                     FROM pg_stat_activity \
                     WHERE application_name = $1 AND state = 'active' \
                     ORDER BY query_start DESC LIMIT 1",
                    [application_name.clone().into()],
                ))
                .await
                .expect("observe authentication connection")
                .is_some_and(|row| row.try_get::<bool>("", "waiting").unwrap_or(false));
            if waiting {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("authentication must wait for the authorization row lock");

    blocker
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "WITH sampled AS (SELECT clock_timestamp() AS now) \
             UPDATE oidc_authorization_transactions \
             SET created_at = sampled.now - interval '9 minutes 59.8 seconds', \
                 expires_at = sampled.now + interval '0.2 seconds' \
             FROM sampled WHERE id = $1",
            [transaction.id.into()],
        ))
        .await
        .expect("shorten the lifetime while preserving the exact ten-minute interval");
    tokio::time::sleep(Duration::from_millis(300)).await;
    blocker
        .commit()
        .await
        .expect("release the row only after the transaction expires");

    let error = authentication
        .await
        .expect("authentication task must not panic")
        .expect_err("time spent waiting for a row lock must not revive an expired transaction");
    assert_unavailable(&error);
}

// NOTE: AI-generated test
#[tokio::test]
async fn concurrent_authentications_have_exactly_one_winner() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let owner = seed_owner(&app, "concurrent-authenticate").await;
    seed_subject(&database, &owner).await;
    let transaction = create_transaction(&database, &owner, "concurrent-authenticate").await;
    let barrier = Arc::new(Barrier::new(3));

    let authenticate = |database: DatabaseConnection, barrier: Arc<Barrier>| {
        let input = authenticate_input(&transaction);
        tokio::spawn(async move {
            barrier.wait().await;
            let owned = database.begin().await.expect("begin concurrent login");
            let result = OidcAuthorizationTransactions::authenticate_authorization_transaction(
                input, &owned,
            )
            .await;
            match result {
                Ok(model) => {
                    owned.commit().await.expect("commit concurrent login");
                    Ok(model)
                }
                Err(error) => {
                    owned.rollback().await.expect("roll back concurrent login");
                    Err(error)
                }
            }
        })
    };
    let first = authenticate(database.clone(), Arc::clone(&barrier));
    let second = authenticate(database.clone(), Arc::clone(&barrier));
    barrier.wait().await;

    let first = first
        .await
        .expect("first authentication task must not panic");
    let second = second
        .await
        .expect("second authentication task must not panic");
    assert_eq!(
        usize::from(first.is_ok()) + usize::from(second.is_ok()),
        1,
        "the conditional UPDATE must return exactly one concurrent winner"
    );

    let winner = first.or(second).expect("one authentication must win");
    assert_eq!(
        winner.status,
        OidcAuthorizationTransactionStatus::Authenticated
    );
    assert_eq!(winner.revision, transaction.revision + 1);
    assert_eq!(winner.subject_id, Some(SUBJECT_ID));
    assert!(winner.authenticated_at.is_some());
    assert!(winner.terminal_at.is_none());

    let stored = OidcAuthorizationTransactions::find_by_id(transaction.id)
        .one(&database)
        .await
        .expect("read concurrently authenticated transaction")
        .expect("authenticated transaction remains stored");
    assert_eq!(
        stored.status,
        OidcAuthorizationTransactionStatus::Authenticated
    );
    assert_eq!(stored.revision, 1);
    assert_eq!(stored.subject_id, Some(SUBJECT_ID));
    assert!(stored.authenticated_at.is_some());
    assert_eq!(stored.created_at, transaction.created_at);
    assert_eq!(stored.expires_at, transaction.expires_at);
}

const CHALLENGE_ID: Uuid = Uuid::from_u128(0x018fc2ab_bd4e_7c2b_8f3a_2a1c4d5e6f71);

fn challenge_input(
    transaction: &oidc_authorization_transactions::Model,
) -> ChallengeAuthorizationTransactionInput {
    ChallengeAuthorizationTransactionInput {
        id: transaction.id,
        browser_binding_digest: BROWSER_DIGEST,
        csrf_digest: CSRF_DIGEST,
        expected_revision: transaction.revision,
        challenge_id: CHALLENGE_ID,
    }
}

fn authenticate_challenged_input(
    transaction: &oidc_authorization_transactions::Model,
) -> AuthenticateChallengedAuthorizationTransactionInput {
    AuthenticateChallengedAuthorizationTransactionInput {
        id: transaction.id,
        browser_binding_digest: BROWSER_DIGEST,
        csrf_digest: CSRF_DIGEST,
        expected_revision: transaction.revision,
        challenge_id: CHALLENGE_ID,
        subject_id: SUBJECT_ID,
    }
}

async fn challenge_in_owned_transaction(
    database: &DatabaseConnection,
    input: ChallengeAuthorizationTransactionInput,
) -> Result<oidc_authorization_transactions::Model, Error> {
    let transaction = database.begin().await.expect("begin challenge transaction");
    let result =
        OidcAuthorizationTransactions::challenge_authorization_transaction(input, &transaction)
            .await;
    match result {
        Ok(model) => {
            transaction
                .commit()
                .await
                .expect("commit challenge transaction");
            Ok(model)
        }
        Err(error) => {
            transaction
                .rollback()
                .await
                .expect("roll back challenge transaction");
            Err(error)
        }
    }
}

async fn authenticate_challenged_in_owned_transaction(
    database: &DatabaseConnection,
    input: AuthenticateChallengedAuthorizationTransactionInput,
) -> Result<oidc_authorization_transactions::Model, Error> {
    let transaction = database
        .begin()
        .await
        .expect("begin challenged login transaction");
    let result = OidcAuthorizationTransactions::authenticate_challenged_authorization_transaction(
        input,
        &transaction,
    )
    .await;
    match result {
        Ok(model) => {
            transaction
                .commit()
                .await
                .expect("commit challenged login transaction");
            Ok(model)
        }
        Err(error) => {
            transaction
                .rollback()
                .await
                .expect("roll back challenged login transaction");
            Err(error)
        }
    }
}

// NOTE: AI-generated test
#[tokio::test]
async fn challenge_transitions_pending_to_awaiting_challenge_and_binds_challenge() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let owner = seed_owner(&app, "challenge").await;
    let original = create_transaction(&database, &owner, "challenge").await;

    let challenged = challenge_in_owned_transaction(&database, challenge_input(&original))
        .await
        .expect("matching bindings and revision should bind the challenge");
    assert_eq!(
        challenged.status,
        OidcAuthorizationTransactionStatus::AwaitingChallenge
    );
    assert_eq!(challenged.revision, original.revision + 1);
    assert_eq!(challenged.challenge_id, Some(CHALLENGE_ID));
    assert!(challenged.subject_id.is_none());
    assert!(challenged.authenticated_at.is_none());
    assert!(challenged.terminal_at.is_none());
    assert_eq!(challenged.created_at, original.created_at);
    assert_eq!(challenged.expires_at, original.expires_at);
    assert_eq!(challenged.tenant_id, original.tenant_id);
    assert_eq!(challenged.application_id, original.application_id);
    assert_eq!(challenged.oidc_client_id, original.oidc_client_id);
    assert_eq!(
        challenged.browser_binding_digest,
        original.browser_binding_digest
    );
    assert_eq!(challenged.csrf_digest, original.csrf_digest);

    // The login read gate stays pending-only; the challenge read gate deliberately has no
    // status predicate so the caller can map live wrong states to the uniform failure page.
    let login_read_error = assert_unavailable(
        &OidcAuthorizationTransactions::get_login_authorization_transaction(
            original.id,
            owner.tenant_id,
            BROWSER_DIGEST,
            CSRF_DIGEST,
            &database,
        )
        .await
        .expect_err("an awaiting_challenge transaction must not be readable for login"),
    );
    let challenged_read = OidcAuthorizationTransactions::get_challenged_authorization_transaction(
        original.id,
        owner.tenant_id,
        BROWSER_DIGEST,
        CSRF_DIGEST,
        &database,
    )
    .await
    .expect("the challenge read gate must return the live awaiting transaction");
    assert_eq!(challenged_read, challenged);

    let repeat_error = assert_unavailable(
        &challenge_in_owned_transaction(&database, challenge_input(&challenged))
            .await
            .expect_err("challenge binding is not repeatable from a non-pending state"),
    );
    assert_eq!(repeat_error, login_read_error);
    let direct_authenticate_error = assert_unavailable(
        &authenticate_in_owned_transaction(
            &database,
            AuthenticateAuthorizationTransactionInput {
                id: challenged.id,
                browser_binding_digest: BROWSER_DIGEST,
                csrf_digest: CSRF_DIGEST,
                expected_revision: challenged.revision,
                subject_id: SUBJECT_ID,
            },
        )
        .await
        .expect_err("the pending-only authenticate path must not apply to awaiting_challenge"),
    );
    assert_eq!(direct_authenticate_error, login_read_error);

    // The status-free challenge gate also returns a plain pending row; the challenge endpoint
    // maps that to the uniform credential-failure page instead of revealing the state.
    let pending = create_transaction(&database, &owner, "challenge-gate-pending").await;
    let pending_read = OidcAuthorizationTransactions::get_challenged_authorization_transaction(
        pending.id,
        owner.tenant_id,
        BROWSER_DIGEST,
        CSRF_DIGEST,
        &database,
    )
    .await
    .expect("the challenge read gate must return a live pending transaction");
    assert_eq!(pending_read, pending);
}

// NOTE: AI-generated test
#[tokio::test]
async fn challenge_requires_fresh_pending_state_and_matching_bindings() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let owner = seed_owner(&app, "challenge-reject").await;
    let original = create_transaction(&database, &owner, "challenge-reject").await;

    let wrong_browser_gate = assert_unavailable(
        &OidcAuthorizationTransactions::get_challenged_authorization_transaction(
            original.id,
            owner.tenant_id,
            [0x99; 32],
            CSRF_DIGEST,
            &database,
        )
        .await
        .expect_err("wrong browser binding must be rejected before the challenge gate"),
    );
    let wrong_csrf_gate = assert_unavailable(
        &OidcAuthorizationTransactions::get_challenged_authorization_transaction(
            original.id,
            owner.tenant_id,
            BROWSER_DIGEST,
            [0x88; 32],
            &database,
        )
        .await
        .expect_err("wrong CSRF binding must be rejected before the challenge gate"),
    );
    assert_eq!(wrong_csrf_gate, wrong_browser_gate);
    let wrong_tenant_gate = assert_unavailable(
        &OidcAuthorizationTransactions::get_challenged_authorization_transaction(
            original.id,
            Uuid::now_v7(),
            BROWSER_DIGEST,
            CSRF_DIGEST,
            &database,
        )
        .await
        .expect_err("a cross-tenant read must be indistinguishable from absence"),
    );
    assert_eq!(wrong_tenant_gate, wrong_browser_gate);

    let mut wrong_browser_input = challenge_input(&original);
    wrong_browser_input.browser_binding_digest = [0x99; 32];
    let wrong_browser_error = assert_unavailable(
        &challenge_in_owned_transaction(&database, wrong_browser_input)
            .await
            .expect_err("wrong browser binding must not bind a challenge"),
    );
    assert_eq!(wrong_browser_error, wrong_browser_gate);

    let mut wrong_csrf_input = challenge_input(&original);
    wrong_csrf_input.csrf_digest = [0x88; 32];
    let wrong_csrf_error = assert_unavailable(
        &challenge_in_owned_transaction(&database, wrong_csrf_input)
            .await
            .expect_err("wrong CSRF binding must not bind a challenge"),
    );
    assert_eq!(wrong_csrf_error, wrong_browser_gate);

    let mut stale_revision_input = challenge_input(&original);
    stale_revision_input.expected_revision = original.revision + 1;
    let stale_revision_error = assert_unavailable(
        &challenge_in_owned_transaction(&database, stale_revision_input)
            .await
            .expect_err("a stale revision must not bind a challenge"),
    );
    assert_eq!(stale_revision_error, wrong_browser_gate);

    let still_pending = OidcAuthorizationTransactions::find_by_id(original.id)
        .one(&database)
        .await
        .expect("read transaction after rejected challenge attempts")
        .expect("transaction should remain stored");
    assert_eq!(
        still_pending.status,
        OidcAuthorizationTransactionStatus::Pending
    );
    assert_eq!(still_pending.revision, original.revision);
    assert_eq!(still_pending.challenge_id, None);

    let expiring = create_transaction(&database, &owner, "challenge-expired").await;
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "UPDATE oidc_authorization_transactions \
             SET created_at = created_at - interval '11 minutes', \
                 expires_at = expires_at - interval '11 minutes' \
             WHERE id = $1",
            [expiring.id.into()],
        ))
        .await
        .expect("expire transaction while preserving its fixed lifetime");
    let expired_gate_error = assert_unavailable(
        &OidcAuthorizationTransactions::get_challenged_authorization_transaction(
            expiring.id,
            owner.tenant_id,
            BROWSER_DIGEST,
            CSRF_DIGEST,
            &database,
        )
        .await
        .expect_err("database-expired transaction must not pass the challenge gate"),
    );
    assert_eq!(expired_gate_error, wrong_browser_gate);
    let expired_challenge_error = assert_unavailable(
        &challenge_in_owned_transaction(&database, challenge_input(&expiring))
            .await
            .expect_err("database-expired transaction must not bind a challenge"),
    );
    assert_eq!(expired_challenge_error, wrong_browser_gate);

    let cancelled = OidcAuthorizationTransactions::cancel_authorization_transaction(
        original.id,
        BROWSER_DIGEST,
        CSRF_DIGEST,
        original.revision,
        &database,
    )
    .await
    .expect("cancel the original transaction");
    let cancelled_gate_error = assert_unavailable(
        &OidcAuthorizationTransactions::get_challenged_authorization_transaction(
            original.id,
            owner.tenant_id,
            BROWSER_DIGEST,
            CSRF_DIGEST,
            &database,
        )
        .await
        .expect_err("a terminal transaction must not pass the challenge gate"),
    );
    assert_eq!(cancelled_gate_error, wrong_browser_gate);
    let cancelled_challenge_error = assert_unavailable(
        &challenge_in_owned_transaction(&database, challenge_input(&cancelled))
            .await
            .expect_err("a non-pending start state must not bind a challenge"),
    );
    assert_eq!(cancelled_challenge_error, wrong_browser_gate);
}

// NOTE: AI-generated test
#[tokio::test]
async fn challenge_rechecks_expiry_after_waiting_for_the_row_lock() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let owner = seed_owner(&app, "challenge-expiry-lock").await;
    let transaction = create_transaction(&database, &owner, "challenge-expiry-lock").await;

    let blocker = database
        .begin()
        .await
        .expect("begin transaction that holds the authorization row lock");
    blocker
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT id FROM oidc_authorization_transactions WHERE id = $1 FOR UPDATE",
            [transaction.id.into()],
        ))
        .await
        .expect("lock authorization transaction")
        .expect("authorization transaction should exist");

    let application_name = format!("oidc_auth_tx_challenge_{}", Uuid::now_v7().simple());
    let challenge_database = Database::connect(format!(
        "{}&application_name={application_name}",
        app.dsn_with_schema()
    ))
    .await
    .expect("connect dedicated challenge database");
    let input = challenge_input(&transaction);
    let challenge = tokio::spawn(async move {
        let transaction = challenge_database
            .begin()
            .await
            .expect("begin blocked challenge transaction");
        let result =
            OidcAuthorizationTransactions::challenge_authorization_transaction(input, &transaction)
                .await;
        transaction
            .rollback()
            .await
            .expect("roll back blocked challenge transaction");
        result
    });

    timeout(Duration::from_secs(2), async {
        loop {
            let waiting = database
                .query_one_raw(Statement::from_sql_and_values(
                    DatabaseBackend::Postgres,
                    "SELECT wait_event_type = 'Lock' AS waiting \
                     FROM pg_stat_activity \
                     WHERE application_name = $1 AND state = 'active' \
                     ORDER BY query_start DESC LIMIT 1",
                    [application_name.clone().into()],
                ))
                .await
                .expect("observe challenge connection")
                .is_some_and(|row| row.try_get::<bool>("", "waiting").unwrap_or(false));
            if waiting {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("challenge binding must wait for the authorization row lock");

    blocker
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "WITH sampled AS (SELECT clock_timestamp() AS now) \
             UPDATE oidc_authorization_transactions \
             SET created_at = sampled.now - interval '9 minutes 59.8 seconds', \
                 expires_at = sampled.now + interval '0.2 seconds' \
             FROM sampled WHERE id = $1",
            [transaction.id.into()],
        ))
        .await
        .expect("shorten the lifetime while preserving the exact ten-minute interval");
    tokio::time::sleep(Duration::from_millis(300)).await;
    blocker
        .commit()
        .await
        .expect("release the row only after the transaction expires");

    let error = challenge
        .await
        .expect("challenge task must not panic")
        .expect_err("time spent waiting for a row lock must not revive an expired transaction");
    assert_unavailable(&error);
}

// NOTE: AI-generated test
#[tokio::test]
async fn concurrent_challenges_have_exactly_one_winner() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let owner = seed_owner(&app, "concurrent-challenge").await;
    let transaction = create_transaction(&database, &owner, "concurrent-challenge").await;
    let barrier = Arc::new(Barrier::new(3));

    let challenge = |database: DatabaseConnection, barrier: Arc<Barrier>| {
        let input = challenge_input(&transaction);
        tokio::spawn(async move {
            barrier.wait().await;
            challenge_in_owned_transaction(&database, input).await
        })
    };
    let first = challenge(database.clone(), Arc::clone(&barrier));
    let second = challenge(database.clone(), Arc::clone(&barrier));
    barrier.wait().await;

    let first = first.await.expect("first challenge task must not panic");
    let second = second.await.expect("second challenge task must not panic");
    assert_eq!(
        usize::from(first.is_ok()) + usize::from(second.is_ok()),
        1,
        "the conditional UPDATE must return exactly one concurrent winner"
    );

    let winner = first.or(second).expect("one challenge binding must win");
    assert_eq!(
        winner.status,
        OidcAuthorizationTransactionStatus::AwaitingChallenge
    );
    assert_eq!(winner.revision, transaction.revision + 1);
    assert_eq!(winner.challenge_id, Some(CHALLENGE_ID));
    assert!(winner.terminal_at.is_none());

    let stored = OidcAuthorizationTransactions::find_by_id(transaction.id)
        .one(&database)
        .await
        .expect("read concurrently challenged transaction")
        .expect("challenged transaction remains stored");
    assert_eq!(
        stored.status,
        OidcAuthorizationTransactionStatus::AwaitingChallenge
    );
    assert_eq!(stored.revision, 1);
    assert_eq!(stored.challenge_id, Some(CHALLENGE_ID));
    assert_eq!(stored.created_at, transaction.created_at);
    assert_eq!(stored.expires_at, transaction.expires_at);
}

// NOTE: AI-generated test
#[tokio::test]
async fn authenticate_challenged_transitions_and_records_subject() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let owner = seed_owner(&app, "authenticate-challenged").await;
    seed_subject(&database, &owner).await;
    let original = create_transaction(&database, &owner, "authenticate-challenged").await;
    let challenged = challenge_in_owned_transaction(&database, challenge_input(&original))
        .await
        .expect("bind the challenge before the challenged authentication");

    let authenticated = authenticate_challenged_in_owned_transaction(
        &database,
        authenticate_challenged_input(&challenged),
    )
    .await
    .expect("matching challenge binding should authenticate");
    assert_eq!(
        authenticated.status,
        OidcAuthorizationTransactionStatus::Authenticated
    );
    assert_eq!(authenticated.revision, challenged.revision + 1);
    assert_eq!(authenticated.subject_id, Some(SUBJECT_ID));
    assert_eq!(authenticated.challenge_id, Some(CHALLENGE_ID));
    let authenticated_at = authenticated
        .authenticated_at
        .expect("challenged authentication must record the database clock");
    assert!(authenticated_at >= authenticated.created_at);
    assert!(authenticated_at < authenticated.expires_at);
    assert!(authenticated.terminal_at.is_none());

    let repeat_error = assert_unavailable(
        &authenticate_challenged_in_owned_transaction(
            &database,
            authenticate_challenged_input(&authenticated),
        )
        .await
        .expect_err("challenged authentication is terminal and cannot be repeated"),
    );
    let mut mismatched = authenticate_challenged_input(&challenged);
    mismatched.challenge_id = Uuid::now_v7();
    let mismatch_error = assert_unavailable(
        &authenticate_challenged_in_owned_transaction(&database, mismatched)
            .await
            .expect_err("a foreign challenge_id must not authenticate"),
    );
    assert_eq!(mismatch_error, repeat_error);

    // The challenge read gate still returns the live authenticated row; the endpoint maps it to
    // the uniform failure page without revealing the state-machine position.
    let gate_read = OidcAuthorizationTransactions::get_challenged_authorization_transaction(
        original.id,
        owner.tenant_id,
        BROWSER_DIGEST,
        CSRF_DIGEST,
        &database,
    )
    .await
    .expect("the challenge read gate must return the live authenticated transaction");
    assert_eq!(gate_read, authenticated);

    let stored = OidcAuthorizationTransactions::find_by_id(original.id)
        .one(&database)
        .await
        .expect("read challenged-authenticated transaction directly")
        .expect("authentication must not delete the row");
    assert_eq!(stored, authenticated);
}

// NOTE: AI-generated test
#[tokio::test]
async fn authenticate_challenged_requires_fresh_awaiting_state_and_matching_bindings() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let owner = seed_owner(&app, "authenticate-challenged-reject").await;
    seed_subject(&database, &owner).await;
    let original = create_transaction(&database, &owner, "authenticate-challenged-reject").await;
    let challenged = challenge_in_owned_transaction(&database, challenge_input(&original))
        .await
        .expect("bind the challenge before the rejection matrix");

    let pending_start = create_transaction(&database, &owner, "challenged-pending-start").await;
    let pending_start_error = assert_unavailable(
        &authenticate_challenged_in_owned_transaction(
            &database,
            authenticate_challenged_input(&pending_start),
        )
        .await
        .expect_err("a pending start state must not challenged-authenticate"),
    );

    let mut wrong_browser_input = authenticate_challenged_input(&challenged);
    wrong_browser_input.browser_binding_digest = [0x99; 32];
    let wrong_browser_error = assert_unavailable(
        &authenticate_challenged_in_owned_transaction(&database, wrong_browser_input)
            .await
            .expect_err("wrong browser binding must not challenged-authenticate"),
    );
    assert_eq!(wrong_browser_error, pending_start_error);

    let mut wrong_csrf_input = authenticate_challenged_input(&challenged);
    wrong_csrf_input.csrf_digest = [0x88; 32];
    let wrong_csrf_error = assert_unavailable(
        &authenticate_challenged_in_owned_transaction(&database, wrong_csrf_input)
            .await
            .expect_err("wrong CSRF binding must not challenged-authenticate"),
    );
    assert_eq!(wrong_csrf_error, pending_start_error);

    let mut stale_revision_input = authenticate_challenged_input(&challenged);
    stale_revision_input.expected_revision = challenged.revision + 1;
    let stale_revision_error = assert_unavailable(
        &authenticate_challenged_in_owned_transaction(&database, stale_revision_input)
            .await
            .expect_err("a stale revision must not challenged-authenticate"),
    );
    assert_eq!(stale_revision_error, pending_start_error);

    let expiring = create_transaction(&database, &owner, "challenged-expired").await;
    let expiring = challenge_in_owned_transaction(&database, challenge_input(&expiring))
        .await
        .expect("bind a challenge before expiring the transaction");
    database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "UPDATE oidc_authorization_transactions \
             SET created_at = created_at - interval '11 minutes', \
                 expires_at = expires_at - interval '11 minutes' \
             WHERE id = $1",
            [expiring.id.into()],
        ))
        .await
        .expect("expire transaction while preserving its fixed lifetime");
    let expired_error = assert_unavailable(
        &authenticate_challenged_in_owned_transaction(
            &database,
            authenticate_challenged_input(&expiring),
        )
        .await
        .expect_err("database-expired transaction must not challenged-authenticate"),
    );
    assert_eq!(expired_error, pending_start_error);

    let still_awaiting = OidcAuthorizationTransactions::find_by_id(challenged.id)
        .one(&database)
        .await
        .expect("read transaction after rejected challenged attempts")
        .expect("transaction should remain stored");
    assert_eq!(
        still_awaiting.status,
        OidcAuthorizationTransactionStatus::AwaitingChallenge
    );
    assert_eq!(still_awaiting.revision, challenged.revision);
    assert_eq!(still_awaiting.challenge_id, Some(CHALLENGE_ID));
    assert_eq!(still_awaiting.subject_id, None);
    assert_eq!(still_awaiting.authenticated_at, None);
}

// NOTE: AI-generated test
#[tokio::test]
async fn concurrent_challenged_authentications_have_exactly_one_winner() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let owner = seed_owner(&app, "concurrent-challenged-authenticate").await;
    seed_subject(&database, &owner).await;
    let transaction =
        create_transaction(&database, &owner, "concurrent-challenged-authenticate").await;
    let challenged = challenge_in_owned_transaction(&database, challenge_input(&transaction))
        .await
        .expect("bind the challenge before the concurrent authentication");
    let barrier = Arc::new(Barrier::new(3));

    let authenticate = |database: DatabaseConnection, barrier: Arc<Barrier>| {
        let input = authenticate_challenged_input(&challenged);
        tokio::spawn(async move {
            barrier.wait().await;
            authenticate_challenged_in_owned_transaction(&database, input).await
        })
    };
    let first = authenticate(database.clone(), Arc::clone(&barrier));
    let second = authenticate(database.clone(), Arc::clone(&barrier));
    barrier.wait().await;

    let first = first
        .await
        .expect("first challenged authentication task must not panic");
    let second = second
        .await
        .expect("second challenged authentication task must not panic");
    assert_eq!(
        usize::from(first.is_ok()) + usize::from(second.is_ok()),
        1,
        "the conditional UPDATE must return exactly one concurrent winner"
    );

    let winner = first
        .or(second)
        .expect("one challenged authentication must win");
    assert_eq!(
        winner.status,
        OidcAuthorizationTransactionStatus::Authenticated
    );
    assert_eq!(winner.revision, challenged.revision + 1);
    assert_eq!(winner.subject_id, Some(SUBJECT_ID));
    assert_eq!(winner.challenge_id, Some(CHALLENGE_ID));
    assert!(winner.authenticated_at.is_some());
    assert!(winner.terminal_at.is_none());

    let stored = OidcAuthorizationTransactions::find_by_id(transaction.id)
        .one(&database)
        .await
        .expect("read concurrently challenged-authenticated transaction")
        .expect("authenticated transaction remains stored");
    assert_eq!(
        stored.status,
        OidcAuthorizationTransactionStatus::Authenticated
    );
    assert_eq!(stored.revision, 2);
    assert_eq!(stored.subject_id, Some(SUBJECT_ID));
    assert!(stored.authenticated_at.is_some());
    assert_eq!(stored.created_at, transaction.created_at);
    assert_eq!(stored.expires_at, transaction.expires_at);
}
