use crate::support::{TestApp, spawn_app_with_isolated_schema};
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use serde_json::json;
use uuid::Uuid;

async fn create_bound_secret(app: &TestApp, token: &str) -> (String, String, String, String) {
    let tenant = app.api_create_tenant(token).await;
    let tenant_id = tenant["id"].as_str().unwrap().to_owned();
    let application = app.api_create_application(token, &tenant_id).await;
    let application_id = application["application_id"].as_str().unwrap().to_owned();

    let created: serde_json::Value = app
        .client
        .post(app.url("/secrets"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let secret_id = created["id"].as_str().unwrap().to_owned();
    let plaintext = created["secret"].as_str().unwrap().to_owned();

    let response = app
        .client
        .post(app.url(&format!("/secrets/{secret_id}/bindings")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "application_id": application_id }))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());

    (tenant_id, application_id, secret_id, plaintext)
}

/// POST /secrets — create a new secret (no request body needed)
/// Asserts: returns 200 with a non-empty `id` and a non-empty `secret` value.
// NOTE: AI-generated test
#[tokio::test]
async fn create_secret_returns_200() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;

    let resp: serde_json::Value = app
        .client
        .post(app.url("/secrets"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("create secret request failed")
        .json()
        .await
        .expect("create secret response parse failed");

    let id = resp["id"].as_str().expect("secret id should be present");
    assert!(!id.is_empty(), "secret id should not be empty");
}

/// POST /secrets → GET /secrets
/// Asserts: the created secret appears in the list.
// NOTE: AI-generated test
#[tokio::test]
async fn get_secrets_returns_created_secret() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;

    // Create a secret
    let create_resp: serde_json::Value = app
        .client
        .post(app.url("/secrets"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("create secret request failed")
        .json()
        .await
        .expect("create secret response parse failed");
    let secret_id = create_resp["id"].as_str().unwrap().to_string();

    // List secrets
    let list: serde_json::Value = app
        .client
        .get(app.url("/secrets"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("list secrets request failed")
        .json()
        .await
        .expect("list secrets response parse failed");

    let items = list["items"].as_array().expect("items should be an array");
    assert!(
        items.iter().any(|s| s["id"].as_str() == Some(&secret_id)),
        "created secret should appear in the list"
    );
}

/// POST /secrets → GET /secrets/{id}
/// Asserts: the returned secret has the matching id.
// NOTE: AI-generated test
#[tokio::test]
async fn get_secret_by_id_returns_200() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;

    let create_resp: serde_json::Value = app
        .client
        .post(app.url("/secrets"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("create secret request failed")
        .json()
        .await
        .expect("create secret response parse failed");
    let secret_id = create_resp["id"].as_str().unwrap().to_string();

    let get_resp: serde_json::Value = app
        .client
        .get(app.url(&format!("/secrets/{secret_id}")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("get secret request failed")
        .json()
        .await
        .expect("get secret response parse failed");

    assert_eq!(
        get_resp["id"].as_str(),
        Some(secret_id.as_str()),
        "returned secret should have matching id"
    );
}

/// POST /secrets → DELETE /secrets/{id}
/// Asserts: delete returns a success status code, and a subsequent GET returns 404.
// NOTE: AI-generated test
#[tokio::test]
async fn create_and_delete_secret() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;

    let create_resp: serde_json::Value = app
        .client
        .post(app.url("/secrets"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("create secret request failed")
        .json()
        .await
        .expect("create secret response parse failed");
    let secret_id = create_resp["id"].as_str().unwrap().to_string();

    let delete_resp = app
        .client
        .delete(app.url(&format!("/secrets/{secret_id}")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("delete secret request failed");

    let status = delete_resp.status();
    assert!(
        status.is_success(),
        "delete secret should return success (got {status})"
    );

    let get_resp = app
        .client
        .get(app.url(&format!("/secrets/{secret_id}")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("get deleted secret request failed");
    assert_eq!(get_resp.status(), 404, "deleted secret should return 404");
}

/// POST /secrets → POST /secrets/{id}/bindings
/// Asserts: binding a secret to an application returns 200, and GET /secrets/{id} shows the binding.
// NOTE: AI-generated test
#[tokio::test]
async fn bind_secret_to_application() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap();
    let app_resp = app.api_create_application(&token, tenant_id).await;
    let application_id = app_resp["application_id"].as_str().unwrap().to_string();

    let create_resp: serde_json::Value = app
        .client
        .post(app.url("/secrets"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("create secret request failed")
        .json()
        .await
        .expect("create secret response parse failed");
    let secret_id = create_resp["id"].as_str().unwrap().to_string();

    let bind_resp = app
        .client
        .post(app.url(&format!("/secrets/{secret_id}/bindings")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "application_id": application_id }))
        .send()
        .await
        .expect("bind secret request failed");

    assert!(
        bind_resp.status().is_success(),
        "bind secret should return success (got {})",
        bind_resp.status()
    );

    let get_resp: serde_json::Value = app
        .client
        .get(app.url(&format!("/secrets/{secret_id}")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("get secret request failed")
        .json()
        .await
        .expect("get secret response parse failed");

    let app_ids = get_resp["application_ids"]
        .as_array()
        .expect("application_ids should be an array");
    assert!(
        app_ids
            .iter()
            .any(|id| id.as_str() == Some(&application_id)),
        "bound application should appear in application_ids"
    );
}

/// POST /secrets → bind → DELETE /secrets/{id}/bindings/{application_id}
/// Asserts: unbinding removes the application from the secret's application_ids.
// NOTE: AI-generated test
#[tokio::test]
async fn unbind_secret_from_application() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap();
    let app_resp = app.api_create_application(&token, tenant_id).await;
    let application_id = app_resp["application_id"].as_str().unwrap().to_string();

    let create_resp: serde_json::Value = app
        .client
        .post(app.url("/secrets"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("create secret request failed")
        .json()
        .await
        .expect("create secret response parse failed");
    let secret_id = create_resp["id"].as_str().unwrap().to_string();

    app.client
        .post(app.url(&format!("/secrets/{secret_id}/bindings")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "application_id": application_id }))
        .send()
        .await
        .expect("bind secret request failed");

    let unbind_resp = app
        .client
        .delete(app.url(&format!("/secrets/{secret_id}/bindings/{application_id}")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("unbind secret request failed");

    assert!(
        unbind_resp.status().is_success(),
        "unbind secret should return success (got {})",
        unbind_resp.status()
    );

    let get_resp: serde_json::Value = app
        .client
        .get(app.url(&format!("/secrets/{secret_id}")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("get secret request failed")
        .json()
        .await
        .expect("get secret response parse failed");

    assert_eq!(
        get_resp["application_ids"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0),
        0,
        "application_ids should be empty after unbind"
    );
}

/// POST /secrets → bind (×2)
/// Asserts: binding an already-bound secret to the same application returns 409 Conflict.
// NOTE: AI-generated test
#[tokio::test]
async fn bind_secret_duplicate_returns_409() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap();
    let app_resp = app.api_create_application(&token, tenant_id).await;
    let application_id = app_resp["application_id"].as_str().unwrap().to_string();

    let create_resp: serde_json::Value = app
        .client
        .post(app.url("/secrets"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("create secret request failed")
        .json()
        .await
        .expect("create secret response parse failed");
    let secret_id = create_resp["id"].as_str().unwrap().to_string();

    app.client
        .post(app.url(&format!("/secrets/{secret_id}/bindings")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "application_id": application_id }))
        .send()
        .await
        .expect("first bind request failed");

    let second_resp = app
        .client
        .post(app.url(&format!("/secrets/{secret_id}/bindings")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "application_id": application_id }))
        .send()
        .await
        .expect("second bind request failed");

    assert_eq!(
        second_resp.status(),
        409,
        "duplicate bind should return 409 Conflict"
    );
}

/// DELETE /secrets/{nonexistent_id}
/// Asserts: deleting a secret that never existed returns 404.
// NOTE: AI-generated test
#[tokio::test]
async fn delete_nonexistent_secret_returns_404() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;

    // Create a secret, delete it, then try to delete the same ID again.
    let create_resp: serde_json::Value = app
        .client
        .post(app.url("/secrets"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("create secret request failed")
        .json()
        .await
        .expect("create secret response parse failed");
    let secret_id = create_resp["id"].as_str().unwrap().to_string();

    app.client
        .delete(app.url(&format!("/secrets/{secret_id}")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("first delete request failed");

    let second_delete = app
        .client
        .delete(app.url(&format!("/secrets/{secret_id}")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("second delete request failed");

    assert_eq!(
        second_delete.status(),
        404,
        "deleting already-deleted secret should return 404"
    );
}

/// POST /secrets → verify unmasked; GET /secrets and GET /secrets/{id} verify masking.
/// Asserts: create returns the full secret, list and get return a masked version.
// NOTE: AI-generated test
#[tokio::test]
async fn create_returns_unmasked_list_and_get_return_masked() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;

    let create_resp: serde_json::Value = app
        .client
        .post(app.url("/secrets"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("create secret request failed")
        .json()
        .await
        .expect("create secret response parse failed");
    let secret_id = create_resp["id"].as_str().unwrap().to_string();
    let full_secret = create_resp["secret"]
        .as_str()
        .expect("secret should be present in create response")
        .to_string();

    assert!(
        full_secret.starts_with("app_"),
        "full secret should start with 'app_', got: {full_secret}"
    );
    assert!(
        full_secret.len() > 20,
        "create response should include the full unmasked secret (len={})",
        full_secret.len()
    );

    let list_resp: serde_json::Value = app
        .client
        .get(app.url("/secrets"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("list secrets request failed")
        .json()
        .await
        .expect("list secrets response parse failed");

    let list_secret = list_resp["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"].as_str() == Some(&secret_id))
        .expect("created secret should appear in list");
    let masked_in_list = list_secret["secret"].as_str().unwrap();

    assert_ne!(
        masked_in_list, full_secret,
        "list response should show masked secret, not the full value"
    );
    assert!(
        masked_in_list.ends_with("..."),
        "masked secret should end with '...', got: {masked_in_list}"
    );

    let get_resp: serde_json::Value = app
        .client
        .get(app.url(&format!("/secrets/{secret_id}")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("get secret request failed")
        .json()
        .await
        .expect("get secret response parse failed");

    let masked_in_get = get_resp["secret"].as_str().unwrap();

    assert_ne!(
        masked_in_get, full_secret,
        "GET response should show masked secret, not the full value"
    );
    assert!(
        masked_in_get.ends_with("..."),
        "masked secret should end with '...', got: {masked_in_get}"
    );
    assert_eq!(
        masked_in_list, masked_in_get,
        "masked value should be consistent between list and get"
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn created_secret_is_stored_only_as_prefix_and_verifier() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let created: serde_json::Value = app
        .client
        .post(app.url("/secrets"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let plaintext = created["secret"].as_str().unwrap();
    let database = app.database().await;

    let columns = database
        .query_all(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT column_name FROM information_schema.columns \
             WHERE table_schema = current_schema() AND table_name = 'application_secrets'"
                .to_owned(),
        ))
        .await
        .unwrap();
    let column_names: Vec<String> = columns
        .iter()
        .map(|row| row.try_get("", "column_name").unwrap())
        .collect();
    assert!(!column_names.iter().any(|column| column == "secret"));

    let row = database
        .query_one(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT secret_prefix, octet_length(secret_verifier) AS verifier_length, \
             hmac_key_version FROM application_secrets WHERE secret_prefix = $1",
            [plaintext[..12].to_owned().into()],
        ))
        .await
        .unwrap()
        .unwrap();
    let prefix: String = row.try_get("", "secret_prefix").unwrap();
    let verifier_length: i32 = row.try_get("", "verifier_length").unwrap();
    let version: i32 = row.try_get("", "hmac_key_version").unwrap();

    assert_eq!(prefix, &plaintext[..12]);
    assert_eq!(verifier_length, 32);
    assert_eq!(version, 1);
}

// NOTE: AI-generated test
#[tokio::test]
async fn application_secret_authenticates_across_all_prefix_candidates() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let (tenant_id, application_id, _, plaintext) = create_bound_secret(&app, &token).await;
    let database = app.database().await;

    database
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "INSERT INTO application_secrets \
             (id, created_at, revoked_at, secret_prefix, secret_verifier, hmac_key_version) \
             VALUES ($1, now(), NULL, $2, $3, 1)",
            vec![
                Uuid::nil().into(),
                plaintext[..12].to_owned().into(),
                vec![0u8; 32].into(),
            ],
        ))
        .await
        .unwrap();

    let valid = app
        .client
        .get(app.url(&format!(
            "/tenants/{tenant_id}/applications/{application_id}/configuration"
        )))
        .header("X-OceanIAM-Application-Secret", &plaintext)
        .send()
        .await
        .unwrap();
    assert_eq!(valid.status(), 200);

    let mut wrong = plaintext.as_bytes().to_vec();
    wrong[12] = if wrong[12] == b'A' { b'B' } else { b'A' };
    let wrong = String::from_utf8(wrong).unwrap();
    let invalid = app
        .client
        .get(app.url(&format!(
            "/tenants/{tenant_id}/applications/{application_id}/configuration"
        )))
        .header("X-OceanIAM-Application-Secret", wrong)
        .send()
        .await
        .unwrap();
    assert_eq!(invalid.status(), 401);

    let malformed = app
        .client
        .get(app.url(&format!(
            "/tenants/{tenant_id}/applications/{application_id}/configuration"
        )))
        .header("X-OceanIAM-Application-Secret", "not-an-app-secret")
        .send()
        .await
        .unwrap();
    assert_eq!(malformed.status(), 401);

    let missing = app
        .client
        .get(app.url(&format!(
            "/tenants/{tenant_id}/applications/{application_id}/configuration"
        )))
        .send()
        .await
        .unwrap();
    assert_eq!(missing.status(), 401);

    let other_application = app.api_create_application(&token, &tenant_id).await;
    let other_application_id = other_application["application_id"].as_str().unwrap();
    let unbound = app
        .client
        .get(app.url(&format!(
            "/tenants/{tenant_id}/applications/{other_application_id}/configuration"
        )))
        .header("X-OceanIAM-Application-Secret", &plaintext)
        .send()
        .await
        .unwrap();
    assert_eq!(unbound.status(), 403);
}

// NOTE: AI-generated test
#[tokio::test]
async fn revoked_secret_is_rejected_immediately() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let (tenant_id, application_id, _, plaintext) = create_bound_secret(&app, &token).await;
    let database = app.database().await;

    database
        .execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "UPDATE application_secrets SET revoked_at = now() WHERE secret_prefix = $1",
            [plaintext[..12].to_owned().into()],
        ))
        .await
        .unwrap();

    let response = app
        .client
        .get(app.url(&format!(
            "/tenants/{tenant_id}/applications/{application_id}/configuration"
        )))
        .header("X-OceanIAM-Application-Secret", plaintext)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 401);
}

// NOTE: AI-generated test
#[tokio::test]
async fn deleted_secret_is_rejected_immediately() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let (tenant_id, application_id, secret_id, plaintext) = create_bound_secret(&app, &token).await;

    let deleted = app
        .client
        .delete(app.url(&format!("/secrets/{secret_id}")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert!(deleted.status().is_success());

    let response = app
        .client
        .get(app.url(&format!(
            "/tenants/{tenant_id}/applications/{application_id}/configuration"
        )))
        .header("X-OceanIAM-Application-Secret", plaintext)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 401);
}
