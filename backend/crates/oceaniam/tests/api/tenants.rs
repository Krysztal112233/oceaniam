use crate::support::spawn_app_with_isolated_schema;

/// Tests `POST /tenants`.
///
/// Creates a tenant via the API, expects 200 + non-empty `id` in the response body.
// NOTE: AI-generated test
#[tokio::test]
async fn create_tenant_returns_200() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let resp = app.api_create_tenant(&token).await;

    let id = resp["id"].as_str().expect("tenant id should be present");
    assert!(!id.is_empty(), "tenant id should not be empty");
}

/// Tests `GET /tenants`.
///
/// Creates a tenant, then lists all tenants and asserts the created tenant appears in the list.
// NOTE: AI-generated test
#[tokio::test]
async fn get_tenants_returns_created_tenant() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap().to_string();

    let list: serde_json::Value = app
        .client
        .get(app.url("/tenants"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("get tenants request failed")
        .json()
        .await
        .expect("get tenants response parse failed");

    let items = list["items"].as_array().expect("items should be an array");
    assert!(
        items.iter().any(|t| t["id"].as_str() == Some(&tenant_id)),
        "created tenant should appear in the list"
    );
}

/// Tests `DELETE /tenants/{tenant_id}`.
///
/// Creates a tenant, deletes it, and asserts the HTTP response is a success status code.
// NOTE: AI-generated test
#[tokio::test]
async fn create_and_delete_tenant() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap().to_string();

    let delete_resp = app.api_delete_tenant(&token, &tenant_id).await;
    assert!(
        delete_resp.status().is_success(),
        "delete tenant should return success"
    );
}
