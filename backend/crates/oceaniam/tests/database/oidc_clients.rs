use chrono::{DateTime, FixedOffset, Utc};
use oceaniam_database::model::{
    applications, oidc_client_redirect_uris, oidc_clients,
    sea_orm_active_enums::{OidcApplicationType, OidcClientType},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ConnectionTrait, DatabaseBackend, EntityTrait, Statement,
};
use uuid::Uuid;

use crate::support::spawn_app_with_isolated_schema;

fn oidc_client(
    id: Uuid,
    application_id: Uuid,
    client_id: &str,
    now: DateTime<FixedOffset>,
) -> oidc_clients::ActiveModel {
    oidc_clients::ActiveModel {
        id: Set(id),
        application_id: Set(application_id),
        client_id: Set(client_id.to_owned()),
        name: Set("Test web client".to_owned()),
        client_type: Set(OidcClientType::Public),
        application_type: Set(OidcApplicationType::Web),
        created_at: Set(now),
    }
}

fn redirect_uri(
    id: Uuid,
    oidc_client_id: Uuid,
    value: &str,
    now: DateTime<FixedOffset>,
) -> oidc_client_redirect_uris::ActiveModel {
    oidc_client_redirect_uris::ActiveModel {
        id: Set(id),
        oidc_client_id: Set(oidc_client_id),
        redirect_uri: Set(value.to_owned()),
        created_at: Set(now),
    }
}

// NOTE: AI-generated test
#[tokio::test]
async fn oidc_client_schema_enforces_v1_constraints_and_cascade_deletion() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let (_tenant_id, application_id) = app.seed_tenant_and_application().await;
    let now: DateTime<FixedOffset> = Utc::now().into();

    let client_id = Uuid::now_v7();
    oidc_client(client_id, application_id, "public-client", now)
        .insert(&database)
        .await
        .expect("valid public web client should insert");

    let error = database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO oidc_clients \
             (id, application_id, client_id, name, client_type, application_type, created_at) \
             VALUES ($1, $2, 'confidential-client', 'Invalid client', \
                     'confidential', 'web', now())",
            vec![Uuid::now_v7().into(), application_id.into()],
        ))
        .await
        .expect_err("v1 enum should reject confidential clients");
    assert!(
        error.to_string().contains("oidc_client_type"),
        "enum error should identify oidc_client_type: {error}"
    );

    let error = database
        .execute_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO oidc_clients \
             (id, application_id, client_id, name, client_type, application_type, created_at) \
             VALUES ($1, $2, 'native-client', 'Invalid client', \
                     'public', 'native', now())",
            vec![Uuid::now_v7().into(), application_id.into()],
        ))
        .await
        .expect_err("v1 enum should reject native clients");
    assert!(
        error.to_string().contains("oidc_application_type"),
        "enum error should identify oidc_application_type: {error}"
    );

    let (_second_tenant_id, second_application_id) = app.seed_tenant_and_application().await;
    let duplicate_client_id_error =
        oidc_client(Uuid::now_v7(), second_application_id, "public-client", now)
            .insert(&database)
            .await
            .expect_err("client_id should be globally unique across tenant issuers");
    assert!(
        duplicate_client_id_error
            .to_string()
            .contains("uq_oidc_clients_client_id"),
        "constraint error should identify global client_id uniqueness: {duplicate_client_id_error}"
    );

    let second_client_id = Uuid::now_v7();
    oidc_client(
        second_client_id,
        second_application_id,
        "second-public-client",
        now,
    )
    .insert(&database)
    .await
    .expect("a different client_id should insert");

    let redirect_id = Uuid::now_v7();
    redirect_uri(
        redirect_id,
        client_id,
        "https://client.example/oidc/callback",
        now,
    )
    .insert(&database)
    .await
    .expect("valid redirect URI should insert");

    let duplicate_error = redirect_uri(
        Uuid::now_v7(),
        client_id,
        "https://client.example/oidc/callback",
        now,
    )
    .insert(&database)
    .await
    .expect_err("duplicate redirect URI for one client should fail");
    assert!(
        duplicate_error
            .to_string()
            .contains("uq_oidc_client_redirect_uris_client_uri"),
        "constraint error should identify redirect URI uniqueness: {duplicate_error}"
    );

    let second_redirect_id = Uuid::now_v7();
    redirect_uri(
        second_redirect_id,
        second_client_id,
        "https://client.example/oidc/callback",
        now,
    )
    .insert(&database)
    .await
    .expect("different clients may register the same redirect URI");

    applications::Entity::delete_by_id(application_id)
        .exec(&database)
        .await
        .expect("application deletion should succeed");

    assert!(
        oidc_clients::Entity::find_by_id(client_id)
            .one(&database)
            .await
            .expect("client lookup should succeed")
            .is_none(),
        "application deletion should cascade to OIDC clients"
    );
    assert!(
        oidc_client_redirect_uris::Entity::find_by_id(redirect_id)
            .one(&database)
            .await
            .expect("redirect URI lookup should succeed")
            .is_none(),
        "OIDC client deletion should cascade to redirect URIs"
    );
    assert!(
        oidc_client_redirect_uris::Entity::find_by_id(second_redirect_id)
            .one(&database)
            .await
            .expect("second redirect URI lookup should succeed")
            .is_some(),
        "deleting one application must not affect another application's OIDC client"
    );
}
