use crate::support::spawn_app_with_isolated_schema;

/// Tests `POST /tenants/{tenant_id}/applications`.
///
/// Creates an application under an existing tenant, expects 200 + non-empty `application_id`.
// NOTE: AI-generated test
#[tokio::test]
async fn create_application_returns_200() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap();

    let resp = app.api_create_application(&token, tenant_id).await;

    let application_id = resp["application_id"]
        .as_str()
        .expect("application_id should be present");
    assert!(
        !application_id.is_empty(),
        "application_id should not be empty"
    );
}

/// Tests `GET /tenants/{tenant_id}/applications`.
///
/// Creates an application, then lists applications and asserts it appears in the list.
// NOTE: AI-generated test
#[tokio::test]
async fn get_applications_returns_created_application() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap();

    let app_resp = app.api_create_application(&token, tenant_id).await;
    let application_id = app_resp["application_id"].as_str().unwrap().to_string();

    let list: serde_json::Value = app
        .client
        .get(app.url(&format!("/tenants/{tenant_id}/applications")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("get applications request failed")
        .json()
        .await
        .expect("get applications response parse failed");

    let items = list["items"].as_array().expect("items should be an array");
    assert!(
        items
            .iter()
            .any(|a| a["id"].as_str() == Some(&application_id)),
        "created application should appear in the list"
    );
}

/// Tests `DELETE /tenants/{tenant_id}/applications/{application_id}`.
///
/// Creates an application, deletes it, and asserts the HTTP response is a success status code.
// NOTE: AI-generated test
#[tokio::test]
async fn create_and_delete_application() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap();
    let app_resp = app.api_create_application(&token, tenant_id).await;
    let application_id = app_resp["application_id"].as_str().unwrap().to_string();

    let delete_resp = app
        .api_delete_application(&token, tenant_id, &application_id)
        .await;
    assert!(
        delete_resp.status().is_success(),
        "delete application should return success"
    );
}

/// Tests `GET /tenants/{tenant_id}/.well-known/jwks.json`.
///
/// Public endpoint — no auth required. Returns the tenant's JWK set.
/// Asserts: the response contains a `keys` array with at least one entry.
// NOTE: AI-generated test
#[tokio::test]
async fn get_tenant_jwks_returns_keys() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap();

    // No auth header needed — this is a public endpoint
    let resp: serde_json::Value = app
        .client
        .get(app.url(&format!("/tenants/{tenant_id}/.well-known/jwks.json")))
        .send()
        .await
        .expect("get tenant jwks request failed")
        .json()
        .await
        .expect("get tenant jwks response parse failed");

    let keys = resp["keys"].as_array().expect("keys should be an array");
    assert!(!keys.is_empty(), "jwks should contain at least one key");
}

/// Tests `GET /tenants/{tenant_id}/users`.
///
/// Lists all application users across all applications in a tenant.
/// Asserts: a created application user appears in the response.
// NOTE: AI-generated test
#[tokio::test]
async fn get_tenant_users_returns_created_user() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap();
    let app_resp = app.api_create_application(&token, tenant_id).await;
    let application_id = app_resp["application_id"].as_str().unwrap();
    let user = app.api_create_user(&token, tenant_id, application_id).await;
    let user_id = user["id"].as_str().unwrap().to_string();

    let list: serde_json::Value = app
        .client
        .get(app.url(&format!("/tenants/{tenant_id}/users")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("get tenant users request failed")
        .json()
        .await
        .expect("get tenant users response parse failed");

    let items = list["items"].as_array().expect("items should be an array");
    assert!(
        items.iter().any(|u| u["id"].as_str() == Some(&user_id)),
        "created user should appear in the tenant users list"
    );
}
