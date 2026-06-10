use crate::support::spawn_app_with_isolated_schema;

/// POST /tenants/{tenant_id}/keys — rotate (generate a new) key for a tenant
/// Asserts: returns 200.
// NOTE: AI-generated test
#[tokio::test]
async fn rotate_key_returns_200() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap();

    let resp = app
        .client
        .post(app.url(&format!("/tenants/{tenant_id}/keys")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("rotate key request failed");

    assert!(resp.status().is_success(), "rotate key should return 200");
}

/// POST /tenants/{tenant_id}/keys → GET /tenants/{tenant_id}/keys
/// Asserts: at least one key is listed after rotation.
// NOTE: AI-generated test
#[tokio::test]
async fn get_keys_returns_at_least_one_key() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap();

    let rotate_resp = app
        .client
        .post(app.url(&format!("/tenants/{tenant_id}/keys")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("rotate key request failed");
    assert!(rotate_resp.status().is_success());

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
    assert!(!items.is_empty(), "at least one key should be listed");
}

/// POST → DELETE /tenants/{tenant_id}/keys/{key_id}
/// Asserts: revoke returns success and the key's status is updated to "revoked".
// NOTE: AI-generated test
#[tokio::test]
async fn rotate_and_revoke_key() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap();

    let rotate_resp = app
        .client
        .post(app.url(&format!("/tenants/{tenant_id}/keys")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("rotate key request failed");
    assert!(rotate_resp.status().is_success());

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
    let key_id = list["items"][0]["key_id"].as_str().unwrap().to_string();

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

    let list2: serde_json::Value = app
        .client
        .get(app.url(&format!("/tenants/{tenant_id}/keys")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("list keys request failed")
        .json()
        .await
        .expect("list keys response parse failed");
    let revoked = list2["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|k| k["key_id"].as_str() == Some(&key_id))
        .expect("revoked key should appear in the list");
    assert_eq!(
        revoked["status"].as_str(),
        Some("Revoked"),
        "key status should be 'Revoked'"
    );
}
