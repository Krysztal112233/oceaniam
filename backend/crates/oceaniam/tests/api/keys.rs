use crate::support::spawn_app_with_isolated_schema;

/// POST /tenants/{tenant_id}/keys — rotate (generate) a new key for a tenant
/// Uses an existing tenant; no request body needed besides the path param.
/// Asserts: returns 200 with a non-empty `key.key_id`.
// NOTE: AI-generated test
#[tokio::test]
async fn rotate_key_returns_200() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap();

    let resp: serde_json::Value = app
        .client
        .post(app.url(&format!("/tenants/{tenant_id}/keys")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("rotate key request failed")
        .json()
        .await
        .expect("rotate key response parse failed");

    let key_id = resp["key"]["key_id"]
        .as_str()
        .expect("key_id should be present");
    assert!(!key_id.is_empty(), "key_id should not be empty");
}

/// POST /tenants/{tenant_id}/keys → GET /tenants/{tenant_id}/keys
/// Asserts: the rotated key appears in the key list.
// NOTE: AI-generated test
#[tokio::test]
async fn get_keys_returns_rotated_key() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap();

    // Rotate a key
    let rotate_resp: serde_json::Value = app
        .client
        .post(app.url(&format!("/tenants/{tenant_id}/keys")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("rotate key request failed")
        .json()
        .await
        .expect("rotate key response parse failed");
    let key_id = rotate_resp["key"]["key_id"].as_str().unwrap().to_string();

    // List keys
    let list: serde_json::Value = app
        .client
        .get(app.url(&format!("/tenants/{tenant_id}/keys")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("list keys request failed")
        .json()
        .await
        .expect("list keys response parse failed");

    let items = list["items"].as_array().expect("items should be an array");
    assert!(
        items.iter().any(|k| k["key_id"].as_str() == Some(&key_id)),
        "rotated key should appear in the list"
    );
}

/// POST /tenants/{tenant_id}/keys → DELETE /tenants/{tenant_id}/keys/{key_id}
/// Asserts: revoke returns a success status code.
// NOTE: AI-generated test
#[tokio::test]
async fn rotate_and_revoke_key() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap();

    let rotate_resp: serde_json::Value = app
        .client
        .post(app.url(&format!("/tenants/{tenant_id}/keys")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("rotate key request failed")
        .json()
        .await
        .expect("rotate key response parse failed");
    let key_id = rotate_resp["key"]["key_id"].as_str().unwrap().to_string();

    let delete_resp = app
        .client
        .delete(app.url(&format!("/tenants/{tenant_id}/keys/{key_id}")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("revoke key request failed");

    assert!(
        delete_resp.status().is_success(),
        "revoke key should return success"
    );
}
