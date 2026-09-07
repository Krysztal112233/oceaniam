use std::time::Duration;

use oceaniam::app::build_openapi_spec;
use oceaniam_common::{consts::SYSTEM_TENANT_UUID, sqid::Sqid};
use oceaniam_database::{
    helper::{applications::ApplicationHelper, oidc_clients::OidcClientsHelper},
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

fn oidc_client_path(tenant_id: &str, application_id: &str, client_id: &str) -> String {
    format!(
        "{}/{client_id}",
        oidc_clients_path(tenant_id, application_id)
    )
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

async fn patch_oidc_client(
    app: &TestApp,
    token: &str,
    tenant_id: &str,
    application_id: &str,
    client_id: &str,
    body: Value,
) -> Response {
    app.client
        .patch(app.url(&oidc_client_path(tenant_id, application_id, client_id)))
        .header("Authorization", format!("Bearer {token}"))
        .json(&body)
        .send()
        .await
        .expect("OIDC client patch request failed")
}

async fn delete_oidc_client(
    app: &TestApp,
    token: &str,
    tenant_id: &str,
    application_id: &str,
    client_id: &str,
) -> Response {
    app.client
        .delete(app.url(&oidc_client_path(tenant_id, application_id, client_id)))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("OIDC client deletion request failed")
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

async fn wait_for_oidc_audit(
    app: &TestApp,
    audit_type: AuditType,
    client_id: &str,
    changed_fields: Option<&[&str]>,
) -> Value {
    let database = app.database().await;
    timeout(Duration::from_secs(2), async {
        loop {
            let audits = Audits::find()
                .filter(oceaniam_database::model::audits::Column::AuditType.eq(audit_type.clone()))
                .all(&database)
                .await
                .expect("audit query should succeed");
            if let Some(audit) = audits.into_iter().find(|audit| {
                if audit.payload["data"]["client_id"].as_str() != Some(client_id) {
                    return false;
                }
                changed_fields.is_none_or(|expected| {
                    audit.payload["data"]["changed_fields"]
                        .as_array()
                        .is_some_and(|actual| {
                            actual
                                == &expected
                                    .iter()
                                    .map(|field| Value::String((*field).to_owned()))
                                    .collect::<Vec<_>>()
                        })
                })
            }) {
                return audit.payload;
            }
            sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .expect("OIDC client audit should be persisted")
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

    let audit = wait_for_oidc_audit(&app, AuditType::CreateOidcClient, &client_id, None).await;
    assert_eq!(audit["kind"], "create_oidc_client");
    assert_eq!(
        audit["data"]["application_id"],
        persisted.application_id.to_string()
    );
    assert_eq!(audit["data"]["client_id"], client_id);

    // Management handlers must query the public identifier as an opaque string rather than
    // decoding it as a Sqid. Percent-encoded delimiters still belong to this one path segment.
    let opaque_internal_id = Uuid::now_v7();
    let opaque_client_id = "client?revision=1/part#%2F";
    OidcClients::create_client(
        opaque_internal_id,
        persisted.application_id,
        opaque_client_id.to_owned(),
        "Opaque identifier client".to_owned(),
        &database,
    )
    .await
    .expect("opaque test OIDC client should insert");
    OidcClients::create_redirect_uris(
        opaque_internal_id,
        vec!["https://opaque.example/callback".to_owned()],
        &database,
    )
    .await
    .expect("opaque test redirect URI should insert");
    let opaque_detail: Value = app
        .client
        .get(app.url(&format!(
            "{}/client%3Frevision%3D1%2Fpart%23%252F",
            oidc_clients_path(tenant_id, application_id)
        )))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("opaque OIDC client detail request failed")
        .error_for_status()
        .expect("opaque OIDC client detail should succeed")
        .json()
        .await
        .expect("opaque OIDC client detail should be JSON");
    assert_eq!(opaque_detail["client_id"], opaque_client_id);
}

// NOTE: AI-generated test
#[tokio::test]
async fn patch_and_delete_oidc_client_are_scoped_atomic_and_audited() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap();
    let application = app.api_create_application(&token, tenant_id).await;
    let application_id = application["application_id"].as_str().unwrap();
    let other_application = app.api_create_application(&token, tenant_id).await;
    let other_application_id = other_application["application_id"].as_str().unwrap();

    let created: Value = create_oidc_client(
        &app,
        &token,
        tenant_id,
        application_id,
        "Original client",
        &["https://original.example/callback"],
    )
    .await
    .error_for_status()
    .expect("OIDC client creation should succeed")
    .json()
    .await
    .expect("OIDC client creation response should be JSON");
    let client_id = created["client_id"].as_str().unwrap().to_owned();
    let oidc_client_id = sqid_to_uuid(&client_id);

    let name_only: Value = patch_oidc_client(
        &app,
        &token,
        tenant_id,
        application_id,
        &client_id,
        json!({ "name": "Renamed client" }),
    )
    .await
    .error_for_status()
    .expect("name-only OIDC client patch should succeed")
    .json()
    .await
    .expect("name-only patch response should be JSON");
    assert_eq!(name_only["name"], "Renamed client");
    assert_eq!(
        name_only["redirect_uris"],
        json!(["https://original.example/callback"])
    );
    for immutable_field in [
        "client_id",
        "application_id",
        "client_type",
        "application_type",
        "created_at",
    ] {
        assert_eq!(
            name_only[immutable_field], created[immutable_field],
            "{immutable_field} must remain immutable"
        );
    }

    let raw_redirect_uris = [
        "https://CLIENT.example/callback/%2Fraw",
        "https://client.example/callback/%2fraw",
    ];
    let uri_only: Value = patch_oidc_client(
        &app,
        &token,
        tenant_id,
        application_id,
        &client_id,
        json!({ "redirect_uris": raw_redirect_uris }),
    )
    .await
    .error_for_status()
    .expect("redirect-URI-only OIDC client patch should succeed")
    .json()
    .await
    .expect("redirect-URI-only patch response should be JSON");
    assert_eq!(uri_only["name"], "Renamed client");
    assert_eq!(uri_only["redirect_uris"], json!(raw_redirect_uris));
    assert!(
        !uri_only["redirect_uris"]
            .as_array()
            .unwrap()
            .contains(&json!("https://original.example/callback")),
        "redirect URI patch must replace the whole set"
    );

    let no_op: Value = patch_oidc_client(
        &app,
        &token,
        tenant_id,
        application_id,
        &client_id,
        json!({}),
    )
    .await
    .error_for_status()
    .expect("empty OIDC client patch should succeed")
    .json()
    .await
    .expect("empty patch response should be JSON");
    assert_eq!(no_op, uri_only);

    let ignored_immutable_fields: Value = patch_oidc_client(
        &app,
        &token,
        tenant_id,
        application_id,
        &client_id,
        json!({
            "id": Uuid::now_v7(),
            "client_id": "replacement-client-id",
            "application_id": other_application_id,
            "client_type": "confidential",
            "application_type": "native",
            "created_at": "2000-01-01T00:00:00Z"
        }),
    )
    .await
    .error_for_status()
    .expect("unknown immutable fields should follow serde's ignore convention")
    .json()
    .await
    .expect("immutable-field patch response should be JSON");
    assert_eq!(ignored_immutable_fields, uri_only);

    let cross_application_patch = patch_oidc_client(
        &app,
        &token,
        tenant_id,
        other_application_id,
        &client_id,
        json!({ "name": "Cross-application write" }),
    )
    .await;
    assert_eq!(cross_application_patch.status(), 404);
    let cross_application_delete =
        delete_oidc_client(&app, &token, tenant_id, other_application_id, &client_id).await;
    assert_eq!(cross_application_delete.status(), 404);

    let name_audit = wait_for_oidc_audit(
        &app,
        AuditType::PatchOidcClient,
        &client_id,
        Some(&["name"]),
    )
    .await;
    assert_eq!(name_audit["kind"], "patch_oidc_client");
    assert_eq!(
        name_audit["data"]["oidc_client_id"],
        oidc_client_id.to_string()
    );
    assert_eq!(name_audit["data"]["name"], "Renamed client");

    let uri_audit = wait_for_oidc_audit(
        &app,
        AuditType::PatchOidcClient,
        &client_id,
        Some(&["redirect_uris"]),
    )
    .await;
    assert_eq!(uri_audit["data"]["name"], "Renamed client");
    assert!(uri_audit["data"].get("redirect_uris").is_none());
    assert!(
        !uri_audit.to_string().contains("CLIENT.example"),
        "patch audit must not contain redirect URI values"
    );

    let no_op_audit =
        wait_for_oidc_audit(&app, AuditType::PatchOidcClient, &client_id, Some(&[])).await;
    assert_eq!(no_op_audit["data"]["changed_fields"], json!([]));

    let delete_response =
        delete_oidc_client(&app, &token, tenant_id, application_id, &client_id).await;
    assert_eq!(delete_response.status(), 204);
    assert!(
        delete_response
            .bytes()
            .await
            .expect("delete response body should be readable")
            .is_empty(),
        "204 response must not have a body"
    );

    let database = app.database().await;
    assert!(
        OidcClients::find_by_id(oidc_client_id)
            .one(&database)
            .await
            .expect("deleted OIDC client lookup should succeed")
            .is_none()
    );
    assert_eq!(
        OidcClientRedirectUris::find()
            .filter(
                oceaniam_database::model::oidc_client_redirect_uris::Column::OidcClientId
                    .eq(oidc_client_id)
            )
            .count(&database)
            .await
            .expect("deleted OIDC client redirect URI count should succeed"),
        0,
        "parent deletion must cascade to redirect URI rows"
    );

    assert_eq!(
        delete_oidc_client(&app, &token, tenant_id, application_id, &client_id)
            .await
            .status(),
        404,
        "repeated deletion must report not found"
    );
    assert_eq!(
        delete_oidc_client(
            &app,
            &token,
            tenant_id,
            application_id,
            "opaque-missing-client-id",
        )
        .await
        .status(),
        404
    );

    let delete_audit =
        wait_for_oidc_audit(&app, AuditType::DeleteOidcClient, &client_id, None).await;
    assert_eq!(delete_audit["kind"], "delete_oidc_client");
    assert_eq!(delete_audit["data"]["name"], "Renamed client");
    assert_eq!(
        delete_audit["data"]["application_id"],
        sqid_to_uuid(application_id).to_string()
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn patch_oidc_client_rejects_invalid_fields_and_rolls_back_all_changes() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap();
    let application = app.api_create_application(&token, tenant_id).await;
    let application_id = application["application_id"].as_str().unwrap();

    let original: Value = create_oidc_client(
        &app,
        &token,
        tenant_id,
        application_id,
        "Original client",
        &["https://original.example/callback"],
    )
    .await
    .error_for_status()
    .expect("OIDC client creation should succeed")
    .json()
    .await
    .expect("OIDC client creation response should be JSON");
    let client_id = original["client_id"].as_str().unwrap();

    let too_many_uris = (0..101)
        .map(|index| format!("https://client-{index}.example/callback"))
        .collect::<Vec<_>>();
    for invalid_patch in [
        json!({ "name": null }),
        json!({ "redirect_uris": null }),
        json!({ "name": "   " }),
        json!({ "name": "x".repeat(129) }),
        json!({ "redirect_uris": [] }),
        json!({ "redirect_uris": too_many_uris }),
        json!({
            "redirect_uris": [
                "https://duplicate.example/callback",
                "https://duplicate.example/callback"
            ]
        }),
        json!({ "redirect_uris": ["http://unsafe.example/callback"] }),
    ] {
        let response = patch_oidc_client(
            &app,
            &token,
            tenant_id,
            application_id,
            client_id,
            invalid_patch,
        )
        .await;
        assert_eq!(response.status(), 400);
    }

    let database = app.database().await;
    database
        .execute_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE oidc_client_redirect_uris \
             ADD CONSTRAINT test_reject_oidc_redirect_uri_patch \
             CHECK (redirect_uri <> 'https://db-failure.example/callback')"
                .to_owned(),
        ))
        .await
        .expect("test redirect rejection constraint should be installed");

    let failed_patch = patch_oidc_client(
        &app,
        &token,
        tenant_id,
        application_id,
        client_id,
        json!({
            "name": "Name that must roll back",
            "redirect_uris": ["https://db-failure.example/callback"]
        }),
    )
    .await;
    assert_eq!(failed_patch.status(), 500);

    let after_failure: Value = app
        .client
        .get(app.url(&oidc_client_path(tenant_id, application_id, client_id)))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("OIDC client lookup after failed patch should succeed")
        .error_for_status()
        .expect("OIDC client should remain after failed patch")
        .json()
        .await
        .expect("OIDC client response should be JSON");
    assert_eq!(after_failure, original, "all patch writes must roll back");

    sleep(Duration::from_millis(100)).await;
    let failed_patch_audit_exists = Audits::find()
        .filter(oceaniam_database::model::audits::Column::AuditType.eq(AuditType::PatchOidcClient))
        .all(&database)
        .await
        .expect("patch audit query should succeed")
        .into_iter()
        .any(|audit| audit.payload["data"]["client_id"] == client_id);
    assert!(
        !failed_patch_audit_exists,
        "rejected and rolled-back patches must not emit audits"
    );
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

    let tenant_b_client: Value = create_oidc_client(
        &app,
        &root_token,
        &tenant_b_id,
        &application_b_id,
        "Tenant B client",
        &["https://tenant-b.example/callback"],
    )
    .await
    .error_for_status()
    .expect("root OIDC client creation should succeed")
    .json()
    .await
    .expect("tenant B OIDC client should be JSON");
    let tenant_b_client_id = tenant_b_client["client_id"].as_str().unwrap();

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
    assert_eq!(
        patch_oidc_client(
            &app,
            &readonly_token,
            &tenant_b_id,
            &application_b_id,
            tenant_b_client_id,
            json!({ "name": "Readonly patch" }),
        )
        .await
        .status(),
        403
    );
    assert_eq!(
        delete_oidc_client(
            &app,
            &readonly_token,
            &tenant_b_id,
            &application_b_id,
            tenant_b_client_id,
        )
        .await
        .status(),
        403
    );

    let tenant_admin_token = create_platform_administrator_token(
        &app,
        &root_token,
        "oidc-tenant-admin",
        PlatformRole::TenantAdmin,
        Some(sqid_to_uuid(&tenant_a_id)),
    )
    .await;
    let assigned_client: Value = create_oidc_client(
        &app,
        &tenant_admin_token,
        &tenant_a_id,
        &application_a_id,
        "Assigned tenant client",
        &["https://assigned.example/callback"],
    )
    .await
    .error_for_status()
    .expect("assigned tenant OIDC client creation should succeed")
    .json()
    .await
    .expect("assigned tenant OIDC client should be JSON");
    let assigned_client_id = assigned_client["client_id"].as_str().unwrap();
    assert_eq!(
        patch_oidc_client(
            &app,
            &tenant_admin_token,
            &tenant_a_id,
            &application_a_id,
            assigned_client_id,
            json!({ "name": "Assigned tenant client patched" }),
        )
        .await
        .status(),
        200
    );
    let unassigned_list = app
        .client
        .get(app.url(&oidc_clients_path(&tenant_b_id, &application_b_id)))
        .header("Authorization", format!("Bearer {tenant_admin_token}"))
        .send()
        .await
        .expect("unassigned tenant OIDC client request failed");
    assert_eq!(unassigned_list.status(), 403);
    assert_eq!(
        patch_oidc_client(
            &app,
            &tenant_admin_token,
            &tenant_b_id,
            &application_b_id,
            tenant_b_client_id,
            json!({ "name": "Unassigned tenant patch" }),
        )
        .await
        .status(),
        403
    );
    assert_eq!(
        delete_oidc_client(
            &app,
            &tenant_admin_token,
            &tenant_b_id,
            &application_b_id,
            tenant_b_client_id,
        )
        .await
        .status(),
        403
    );

    // The previous writes warmed the TenantAdmin permission cache. A role transition must not
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
    assert_eq!(
        patch_oidc_client(
            &app,
            &tenant_admin_token,
            &tenant_b_id,
            &application_b_id,
            tenant_b_client_id,
            json!({ "name": "Cached patch escalation" }),
        )
        .await
        .status(),
        403
    );

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
    assert_eq!(
        patch_oidc_client(
            &app,
            &application_token,
            &tenant_a_id,
            &application_a_id,
            assigned_client_id,
            json!({ "name": "Application token patch" }),
        )
        .await
        .status(),
        400
    );
    assert_eq!(
        delete_oidc_client(
            &app,
            &application_token,
            &tenant_a_id,
            &application_a_id,
            assigned_client_id,
        )
        .await
        .status(),
        400
    );
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
    assert_eq!(
        patch_oidc_client(
            &app,
            &token,
            &tenant_id,
            &application_id,
            "opaque-client-id",
            json!({}),
        )
        .await
        .status(),
        404
    );
    assert_eq!(
        delete_oidc_client(
            &app,
            &token,
            &tenant_id,
            &application_id,
            "opaque-client-id",
        )
        .await
        .status(),
        404
    );
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
    let item = paths
        .get("/tenants/{tenant_id}/applications/{application_id}/oidc-clients/{client_id}")
        .expect("OIDC client detail path should be documented");
    assert!(item.get("get").is_some());
    assert!(item.get("patch").is_some());
    assert!(item.get("delete").is_some());
    assert!(
        item["delete"]["responses"].get("204").is_some(),
        "OIDC client delete 204 response should be documented"
    );

    let patch_schema = &value["components"]["schemas"]["PatchOidcClientRequest"];
    assert!(
        patch_schema.get("required").is_none(),
        "both PATCH fields must remain optional"
    );
    assert_eq!(patch_schema["properties"]["name"]["type"], "string");
    assert_eq!(patch_schema["properties"]["redirect_uris"]["type"], "array");
    assert!(
        patch_schema["properties"]["name"].get("nullable").is_none()
            && patch_schema["properties"]["redirect_uris"]
                .get("nullable")
                .is_none(),
        "explicit null must not be advertised for non-nullable PATCH fields"
    );
}
