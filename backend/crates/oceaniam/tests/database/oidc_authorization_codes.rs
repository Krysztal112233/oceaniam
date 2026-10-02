use chrono::Duration;
use oceaniam_database::{
    Error,
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
use uuid::Uuid;

use crate::support::{TestApp, spawn_app_with_isolated_schema};

const CODE_DIGEST: [u8; 32] = [0x33; 32];
const CODE_TTL: Duration = Duration::seconds(60);

struct Owner {
    transaction_id: Uuid,
}

async fn seed_owner(app: &TestApp, label: &str) -> Owner {
    let database = app.database().await;
    let (tenant_id, application_id) = app.seed_tenant_and_application().await;
    let oidc_client_id = Uuid::now_v7();
    oidc_clients::Entity::create_client(
        oidc_client_id,
        application_id,
        format!("authorization-code-{label}"),
        format!("Authorization code {label}"),
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

    Owner {
        transaction_id: transaction.id,
    }
}

fn issue_input(owner: &Owner, code_digest: [u8; 32]) -> IssueAuthorizationCodeInput {
    IssueAuthorizationCodeInput {
        transaction_id: owner.transaction_id,
        code_digest,
        redirect_uri: "https://client.example/callback".to_owned(),
        subject_id: Uuid::now_v7(),
    }
}

fn assert_unavailable(error: &Error) {
    match error {
        Error::CustomMessage { code, msg, .. } => {
            assert_eq!(*code, 404);
            assert_eq!(msg, "OIDC authorization code is unavailable");
        }
        other => panic!("expected generic unavailable error, got {other}"),
    }
}

// NOTE: AI-generated test
#[tokio::test]
async fn issue_then_consume_roundtrip_anchors_one_clock_sample() {
    let app = spawn_app_with_isolated_schema().await;
    let owner = seed_owner(&app, "roundtrip").await;
    let database = app.database().await;

    let issued = OidcAuthorizationCodes::issue_authorization_code(
        issue_input(&owner, CODE_DIGEST),
        CODE_TTL,
        &database,
    )
    .await
    .expect("issue authorization code");
    assert_eq!(issued.transaction_id, owner.transaction_id);
    assert_eq!(issued.code_digest, CODE_DIGEST.to_vec());
    assert!(issued.redeemed_at.is_none());
    assert_eq!(
        issued.expires_at - issued.created_at,
        CODE_TTL,
        "one database-clock sample must anchor created_at/expires_at exactly"
    );

    let consumed = OidcAuthorizationCodes::consume_authorization_code(CODE_DIGEST, &database)
        .await
        .expect("consume authorization code");
    assert_eq!(consumed.id, issued.id);
    let redeemed_at = consumed.redeemed_at.expect("redeemed_at recorded");
    assert!(redeemed_at >= consumed.created_at);
    assert!(redeemed_at < consumed.expires_at);
}

// NOTE: AI-generated test
#[tokio::test]
async fn consume_is_single_use() {
    let app = spawn_app_with_isolated_schema().await;
    let owner = seed_owner(&app, "single-use").await;
    let database = app.database().await;
    OidcAuthorizationCodes::issue_authorization_code(
        issue_input(&owner, CODE_DIGEST),
        CODE_TTL,
        &database,
    )
    .await
    .expect("issue authorization code");

    OidcAuthorizationCodes::consume_authorization_code(CODE_DIGEST, &database)
        .await
        .expect("first consume succeeds");
    let second = OidcAuthorizationCodes::consume_authorization_code(CODE_DIGEST, &database)
        .await
        .expect_err("second consume must fail");
    assert_unavailable(&second);
}

// NOTE: AI-generated test
#[tokio::test]
async fn expired_code_cannot_be_consumed() {
    let app = spawn_app_with_isolated_schema().await;
    let owner = seed_owner(&app, "expired").await;
    let database = app.database().await;

    let negative_ttl = Duration::seconds(-1);
    let issued = OidcAuthorizationCodes::issue_authorization_code(
        issue_input(&owner, CODE_DIGEST),
        negative_ttl,
        &database,
    )
    .await
    .expect("issue already-expired authorization code");
    assert_eq!(issued.expires_at - issued.created_at, negative_ttl);

    let error = OidcAuthorizationCodes::consume_authorization_code(CODE_DIGEST, &database)
        .await
        .expect_err("an expired code must not be consumable");
    assert_unavailable(&error);
}

// NOTE: AI-generated test
#[tokio::test]
async fn unknown_digest_cannot_be_consumed() {
    let app = spawn_app_with_isolated_schema().await;
    let owner = seed_owner(&app, "unknown").await;
    let database = app.database().await;
    OidcAuthorizationCodes::issue_authorization_code(
        issue_input(&owner, CODE_DIGEST),
        CODE_TTL,
        &database,
    )
    .await
    .expect("issue authorization code");

    let error = OidcAuthorizationCodes::consume_authorization_code([0x44; 32], &database)
        .await
        .expect_err("an unknown digest must not be consumable");
    assert_unavailable(&error);
}
