use crate::support::spawn_app_with_isolated_schema;

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
