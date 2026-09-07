use std::time::Duration;

use oceaniam::app::build_openapi_spec;
use oceaniam_common::{consts::SYSTEM_TENANT_UUID, sqid::Sqid};
use oceaniam_database::{
    helper::applications::ApplicationHelper,
    model::{administrator_tenants, prelude::*, sea_orm_active_enums::AuditType},
};
use oceaniam_permission::PlatformRole;
use reqwest::Response;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseBackend, EntityTrait,
    IntoActiveModel, PaginatorTrait, QueryFilter, Statement,
};
use serde_json::{Value, json};
use tokio::time::{sleep, timeout};
use uuid::Uuid;

use crate::support::{TestApp, spawn_app_with_isolated_schema};

fn oidc_clients_path(tenant_id: &str, application_id: &str) -> String {
    format!("/tenants/{tenant_id}/applications/{application_id}/oidc-clients")
}

async fn create_oidc_client(
    app: &TestApp,
    token: &str,
    tenant_id: &str,
    application_id: &str,
    name: &str,
    redirect_uris: &[&str],
) -> Response {
    app.client
        .post(app.url(&oidc_clients_path(tenant_id, application_id)))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "name": name,
            "redirect_uris": redirect_uris,
        }))
        .send()
        .await
        .expect("OIDC client creation request failed")
}

fn sqid_to_uuid(value: &str) -> Uuid {
    Uuid::try_from(value.parse::<Sqid>().expect("value should be a Sqid"))
        .expect("Sqid should decode to UUID")
}

async fn create_platform_administrator_token(
    app: &TestApp,
    root_token: &str,
    name: &str,
    role: PlatformRole,
    tenant_id: Option<Uuid>,
) -> String {
    let response = app
        .client
        .post(app.url("/administrators"))
        .header("Authorization", format!("Bearer {root_token}"))
        .json(&json!({ "name": name }))
        .send()
        .await
        .expect("administrator creation request failed")
        .error_for_status()
        .expect("administrator creation should succeed");
    let body: Value = response
        .json()
        .await
        .expect("administrator response should be JSON");
    let administrator_id = sqid_to_uuid(
        body["administrator"]["id"]
            .as_str()
            .expect("administrator ID should be present"),
    );
    let password = body["initial_password"]
        .as_str()
        .expect("initial password should be present");

    let database = app.database().await;
    let mut administrator = Administrators::find_by_id(administrator_id)
        .one(&database)
        .await
        .expect("administrator query should succeed")
        .expect("administrator should exist")
        .into_active_model();
    administrator.role = Set(Some(role.to_string()));
    administrator
        .update(&database)
        .await
        .expect("administrator role update should succeed");

    if let Some(tenant_id) = tenant_id {
        administrator_tenants::ActiveModel {
            id: Set(Uuid::now_v7()),
            administrator_id: Set(administrator_id),
            tenant_id: Set(tenant_id),
        }
        .insert(&database)
        .await
        .expect("administrator tenant assignment should succeed");
    }

    let response: Value = app
        .client
        .post(app.url("/auth/tokens"))
        .json(&json!({ "name": name, "password": password }))
        .send()
        .await
        .expect("administrator signin request failed")
        .error_for_status()
        .expect("administrator signin should succeed")
        .json()
        .await
        .expect("administrator signin response should be JSON");

    response["jwt"]
        .as_str()
        .expect("administrator JWT should be present")
        .to_owned()
}

async fn create_application_token(
    app: &TestApp,
    root_token: &str,
    tenant_id: &str,
    application_id: &str,
) -> String {
    let secret: Value = app
        .client
        .post(app.url("/secrets"))
        .header("Authorization", format!("Bearer {root_token}"))
        .send()
        .await
        .expect("application secret creation request failed")
        .error_for_status()
        .expect("application secret creation should succeed")
        .json()
        .await
        .expect("application secret response should be JSON");
    let secret_id = secret["id"].as_str().expect("secret ID should be present");
    let plaintext = secret["secret"]
        .as_str()
        .expect("application secret should be present");

    app.client
        .post(app.url(&format!("/secrets/{secret_id}/bindings")))
        .header("Authorization", format!("Bearer {root_token}"))
        .json(&json!({ "application_id": application_id }))
        .send()
        .await
        .expect("application secret binding request failed")
        .error_for_status()
        .expect("application secret binding should succeed");

    app.api_create_user_with_credentials(
        root_token,
        tenant_id,
        application_id,
        "oidc-client-test@example.com",
        "OidcClientTestPassword123!",
    )
    .await;

    let response: Value = app
        .client
        .post(app.url(&format!(
            "/tenants/{tenant_id}/applications/{application_id}/tokens"
        )))
        .header("X-OceanIAM-Application-Secret", plaintext)
        .json(&json!({
            "email": "oidc-client-test@example.com",
            "password": "OidcClientTestPassword123!",
        }))
        .send()
        .await
        .expect("application signin request failed")
        .error_for_status()
        .expect("application signin should succeed")
        .json()
        .await
        .expect("application signin response should be JSON");

    response["jwt"]
        .as_str()
        .expect("application JWT should be present")
        .to_owned()
}

async fn wait_for_create_audit(app: &TestApp, client_id: &str) -> Value {
    let database = app.database().await;
    timeout(Duration::from_secs(2), async {
        loop {
            let audits = Audits::find()
                .filter(
                    oceaniam_database::model::audits::Column::AuditType
                        .eq(AuditType::CreateOidcClient),
                )
                .all(&database)
                .await
                .expect("audit query should succeed");
            if let Some(audit) = audits
                .into_iter()
                .find(|audit| audit.payload["data"]["client_id"].as_str() == Some(client_id))
            {
                return audit.payload;
            }
            sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .expect("OIDC client creation audit should be persisted")
}

// NOTE: AI-generated test
#[tokio::test]
async fn create_list_and_get_oidc_client_use_client_id_sqid_and_preserve_uris() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap();
    let application = app.api_create_application(&token, tenant_id).await;
    let application_id = application["application_id"].as_str().unwrap();
    let second_application = app.api_create_application(&token, tenant_id).await;
    let second_application_id = second_application["application_id"].as_str().unwrap();
    let redirect_uris = [
        "https://client.example/callback/%2Fencoded",
        "https://CLIENT.example:8443/another-callback",
    ];

    let response = create_oidc_client(
        &app,
        &token,
        tenant_id,
        application_id,
        "Primary web client",
        &redirect_uris,
    )
    .await;
    assert_eq!(response.status(), 200);
    let created: Value = response
        .json()
        .await
        .expect("create response should be JSON");
    let client_id = created["client_id"]
        .as_str()
        .expect("client_id should be present")
        .to_owned();
    let internal_id = sqid_to_uuid(&client_id);

    assert!(
        created.get("id").is_none(),
        "internal ID must not be exposed"
    );
    assert_eq!(created["application_id"], application_id);
    assert_eq!(created["name"], "Primary web client");
    assert_eq!(created["client_type"], "public");
    assert_eq!(created["application_type"], "web");
    let mut returned_uris = created["redirect_uris"]
        .as_array()
        .expect("redirect_uris should be an array")
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    returned_uris.sort_unstable();
    let mut expected_uris = redirect_uris.map(str::to_owned).to_vec();
    expected_uris.sort_unstable();
    assert_eq!(returned_uris, expected_uris);

    let database = app.database().await;
    let persisted = OidcClients::find_by_id(internal_id)
        .one(&database)
        .await
        .expect("OIDC client lookup should succeed")
        .expect("OIDC client should be persisted");
    assert_eq!(persisted.client_id, client_id);
    assert_eq!(persisted.application_id, sqid_to_uuid(application_id));

    let list: Value = app
        .client
        .get(app.url(&format!(
            "{}?page=1&per_page=1",
            oidc_clients_path(tenant_id, application_id)
        )))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("OIDC client list request failed")
        .error_for_status()
        .expect("OIDC client list should succeed")
        .json()
        .await
        .expect("OIDC client list response should be JSON");
    assert_eq!(list["page_info"]["total"], 1);
    assert_eq!(list["page_info"]["has_next"], false);
    assert_eq!(list["items"][0]["client_id"], client_id);

    for partial_query in ["page=1", "per_page=1"] {
        let response = app
            .client
            .get(app.url(&format!(
                "{}?{partial_query}",
                oidc_clients_path(tenant_id, application_id)
            )))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .expect("partial OIDC pagination request failed");
        assert_eq!(
            response.status(),
            200,
            "each optional pagination parameter must work independently"
        );
    }
    let excessive_page = app
        .client
        .get(app.url(&format!(
            "{}?page={}&per_page=100",
            oidc_clients_path(tenant_id, application_id),
            u64::MAX
        )))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("excessive OIDC pagination request failed");
    assert_eq!(excessive_page.status(), 400);

    let detail: Value = app
        .client
        .get(app.url(&format!(
            "{}/{client_id}",
            oidc_clients_path(tenant_id, application_id)
        )))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("OIDC client detail request failed")
        .error_for_status()
        .expect("OIDC client detail should succeed")
        .json()
        .await
        .expect("OIDC client detail response should be JSON");
    assert_eq!(detail, created);

    let wrong_application = app
        .client
        .get(app.url(&format!(
            "{}/{client_id}",
            oidc_clients_path(tenant_id, second_application_id)
        )))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("cross-application OIDC client request failed");
    assert_eq!(wrong_application.status(), 404);

    let second_list: Value = app
        .client
        .get(app.url(&oidc_clients_path(tenant_id, second_application_id)))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("second application OIDC client list request failed")
        .error_for_status()
        .expect("second application OIDC client list should succeed")
        .json()
        .await
        .expect("second application list response should be JSON");
    assert_eq!(second_list["page_info"]["total"], 0);
    assert_eq!(second_list["items"], json!([]));

    let audit = wait_for_create_audit(&app, &client_id).await;
    assert_eq!(audit["kind"], "create_oidc_client");
    assert_eq!(
        audit["data"]["application_id"],
        persisted.application_id.to_string()
    );
    assert_eq!(audit["data"]["client_id"], client_id);
}

// NOTE: AI-generated test
#[tokio::test]
async fn create_oidc_client_enforces_redirect_policy_and_is_atomic() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap();
    let application = app.api_create_application(&token, tenant_id).await;
    let application_id = application["application_id"].as_str().unwrap();

    let missing_redirect_uri = create_oidc_client(
        &app,
        &token,
        tenant_id,
        application_id,
        "No redirect URI",
        &[],
    )
    .await;
    assert_eq!(missing_redirect_uri.status(), 400);

    let blank_name = create_oidc_client(
        &app,
        &token,
        tenant_id,
        application_id,
        "   ",
        &["https://client.example/callback"],
    )
    .await;
    assert_eq!(blank_name.status(), 400);

    let insecure = create_oidc_client(
        &app,
        &token,
        tenant_id,
        application_id,
        "Development client",
        &["http://localhost:3000/callback"],
    )
    .await;
    assert_eq!(insecure.status(), 400);

    let duplicate = create_oidc_client(
        &app,
        &token,
        tenant_id,
        application_id,
        "Duplicate URI client",
        &[
            "https://client.example/callback",
            "https://client.example/callback",
        ],
    )
    .await;
    assert_eq!(duplicate.status(), 400);

    let database = app.database().await;
    let application_uuid = sqid_to_uuid(application_id);
    assert_eq!(
        OidcClients::find()
            .filter(
                oceaniam_database::model::oidc_clients::Column::ApplicationId.eq(application_uuid)
            )
            .count(&database)
            .await
            .expect("OIDC client count should succeed"),
        0,
        "invalid registrations must not leave a parent client row"
    );

    database
        .execute_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE oidc_client_redirect_uris \
             ADD CONSTRAINT test_reject_oidc_redirect_uri \
             CHECK (redirect_uri <> 'https://db-failure.example/callback')"
                .to_owned(),
        ))
        .await
        .expect("test redirect rejection constraint should be installed");
    let database_failure = create_oidc_client(
        &app,
        &token,
        tenant_id,
        application_id,
        "Database failure client",
        &["https://db-failure.example/callback"],
    )
    .await;
    assert_eq!(database_failure.status(), 500);
    assert_eq!(
        OidcClients::find()
            .filter(
                oceaniam_database::model::oidc_clients::Column::ApplicationId.eq(application_uuid)
            )
            .count(&database)
            .await
            .expect("OIDC client count after rollback should succeed"),
        0,
        "a redirect insertion failure must roll back the parent client"
    );
    assert_eq!(
        OidcClientRedirectUris::find()
            .count(&database)
            .await
            .expect("redirect URI count after rollback should succeed"),
        0
    );
    sleep(Duration::from_millis(100)).await;
    let failed_audit_exists = Audits::find()
        .filter(oceaniam_database::model::audits::Column::AuditType.eq(AuditType::CreateOidcClient))
        .all(&database)
        .await
        .expect("audit query after rollback should succeed")
        .into_iter()
        .any(|audit| audit.payload["data"]["name"] == "Database failure client");
    assert!(
        !failed_audit_exists,
        "a rolled-back registration must not emit a creation audit"
    );

    app.client
        .patch(app.url(&format!(
            "/tenants/{tenant_id}/applications/{application_id}/configuration"
        )))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "oidc": { "allow_insecure_loopback_redirect_uris": true }
        }))
        .send()
        .await
        .expect("OIDC policy patch request failed")
        .error_for_status()
        .expect("OIDC policy patch should succeed");

    let accepted = create_oidc_client(
        &app,
        &token,
        tenant_id,
        application_id,
        "Development client",
        &["http://localhost:3000/callback/%2Fraw"],
    )
    .await;
    assert_eq!(accepted.status(), 200);
    let accepted: Value = accepted.json().await.unwrap();
    assert_eq!(
        accepted["redirect_uris"],
        json!(["http://localhost:3000/callback/%2Fraw"]),
        "the registered raw URI must be returned without normalization"
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn oidc_client_permissions_and_tenant_scope_reject_unauthorized_writes() {
    let app = spawn_app_with_isolated_schema().await;
    let root_token = app.root_signin().await;

    let tenant_a = app.api_create_tenant(&root_token).await;
    let tenant_a_id = tenant_a["id"].as_str().unwrap().to_owned();
    let application_a = app.api_create_application(&root_token, &tenant_a_id).await;
    let application_a_id = application_a["application_id"].as_str().unwrap().to_owned();

    let tenant_b = app.api_create_tenant(&root_token).await;
    let tenant_b_id = tenant_b["id"].as_str().unwrap().to_owned();
    let application_b = app.api_create_application(&root_token, &tenant_b_id).await;
    let application_b_id = application_b["application_id"].as_str().unwrap().to_owned();

    create_oidc_client(
        &app,
        &root_token,
        &tenant_b_id,
        &application_b_id,
        "Tenant B client",
        &["https://tenant-b.example/callback"],
    )
    .await
    .error_for_status()
    .expect("root OIDC client creation should succeed");

    let readonly_token = create_platform_administrator_token(
        &app,
        &root_token,
        "oidc-readonly",
        PlatformRole::ReadonlyAdmin,
        None,
    )
    .await;
    let readonly_list = app
        .client
        .get(app.url(&oidc_clients_path(&tenant_b_id, &application_b_id)))
        .header("Authorization", format!("Bearer {readonly_token}"))
        .send()
        .await
        .expect("readonly OIDC client list request failed");
    assert_eq!(readonly_list.status(), 200);
    let readonly_create = create_oidc_client(
        &app,
        &readonly_token,
        &tenant_a_id,
        &application_a_id,
        "Readonly write",
        &["https://readonly.example/callback"],
    )
    .await;
    assert_eq!(readonly_create.status(), 403);

    let tenant_admin_token = create_platform_administrator_token(
        &app,
        &root_token,
        "oidc-tenant-admin",
        PlatformRole::TenantAdmin,
        Some(sqid_to_uuid(&tenant_a_id)),
    )
    .await;
    let assigned_create = create_oidc_client(
        &app,
        &tenant_admin_token,
        &tenant_a_id,
        &application_a_id,
        "Assigned tenant client",
        &["https://assigned.example/callback"],
    )
    .await;
    assert_eq!(assigned_create.status(), 200);
    let unassigned_list = app
        .client
        .get(app.url(&oidc_clients_path(&tenant_b_id, &application_b_id)))
        .header("Authorization", format!("Bearer {tenant_admin_token}"))
        .send()
        .await
        .expect("unassigned tenant OIDC client request failed");
    assert_eq!(unassigned_list.status(), 403);

    // The previous create warmed the TenantAdmin permission cache. A role transition must not
    // combine that cached write activity with ReadonlyAdmin's freshly loaded global scope.
    let database = app.database().await;
    let mut tenant_administrator = Administrators::find()
        .filter(oceaniam_database::model::administrators::Column::Name.eq("oidc-tenant-admin"))
        .one(&database)
        .await
        .expect("tenant administrator query should succeed")
        .expect("tenant administrator should exist")
        .into_active_model();
    tenant_administrator.role = Set(Some(PlatformRole::ReadonlyAdmin.to_string()));
    tenant_administrator
        .update(&database)
        .await
        .expect("tenant administrator role transition should succeed");
    let transitioned_write = create_oidc_client(
        &app,
        &tenant_admin_token,
        &tenant_b_id,
        &application_b_id,
        "Cached permission escalation",
        &["https://cached-permission.example/callback"],
    )
    .await;
    assert_eq!(transitioned_write.status(), 403);

    let application_token =
        create_application_token(&app, &root_token, &tenant_a_id, &application_a_id).await;
    let application_token_request = app
        .client
        .get(app.url(&oidc_clients_path(&tenant_a_id, &application_a_id)))
        .header("Authorization", format!("Bearer {application_token}"))
        .send()
        .await
        .expect("application-token OIDC client request failed");
    assert_eq!(application_token_request.status(), 400);
}

// NOTE: AI-generated test
#[tokio::test]
async fn system_tenant_is_excluded_from_oidc_client_management() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let database = app.database().await;
    let application_id = Uuid::now_v7();
    Applications::create_application(application_id, SYSTEM_TENANT_UUID, &database)
        .await
        .expect("system-tenant test application should insert");
    let tenant_id = Sqid::from(SYSTEM_TENANT_UUID).to_string();
    let application_id = Sqid::from(application_id).to_string();

    let response = app
        .client
        .get(app.url(&oidc_clients_path(&tenant_id, &application_id)))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("system-tenant OIDC client request failed");
    assert_eq!(response.status(), 404);
}

// NOTE: AI-generated test
#[test]
fn openapi_registers_oidc_client_management_paths() {
    let document = build_openapi_spec();
    let value = serde_json::to_value(document).expect("OpenAPI should serialize");
    let paths = value["paths"]
        .as_object()
        .expect("paths should be an object");
    let collection = paths
        .get("/tenants/{tenant_id}/applications/{application_id}/oidc-clients")
        .expect("OIDC client collection path should be documented");
    assert!(collection.get("get").is_some());
    assert!(collection.get("post").is_some());
    assert!(
        paths
            .get("/tenants/{tenant_id}/applications/{application_id}/oidc-clients/{client_id}")
            .and_then(|path| path.get("get"))
            .is_some(),
        "OIDC client detail path should be documented"
    );
}
