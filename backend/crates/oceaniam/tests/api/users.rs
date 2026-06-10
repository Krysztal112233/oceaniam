use crate::support::spawn_app_with_isolated_schema;

/// Tests `POST /tenants/{tenant_id}/applications/{application_id}/users`.
///
/// Creates a user with email + password under an existing application, expects 200 +
/// non-empty `id` and the email matching the request.
// NOTE: AI-generated test
#[tokio::test]
async fn create_user_returns_200() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap();
    let app_resp = app.api_create_application(&token, tenant_id).await;
    let application_id = app_resp["application_id"].as_str().unwrap();

    let resp = app.api_create_user(&token, tenant_id, application_id).await;

    let user_id = resp["id"].as_str().expect("user id should be present");
    assert!(!user_id.is_empty(), "user id should not be empty");
    assert_eq!(
        resp["email"].as_str(),
        Some("test@example.com"),
        "user email should match"
    );
}

/// Tests `GET /tenants/{tenant_id}/applications/{application_id}/users`.
///
/// Creates a user, then lists users and asserts the created user appears in the list.
// NOTE: AI-generated test
#[tokio::test]
async fn get_users_returns_created_user() {
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
        .get(app.url(&format!(
            "/tenants/{tenant_id}/applications/{application_id}/users"
        )))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("get users request failed")
        .json()
        .await
        .expect("get users response parse failed");

    let items = list["items"].as_array().expect("items should be an array");
    assert!(
        items.iter().any(|u| u["id"].as_str() == Some(&user_id)),
        "created user should appear in the list"
    );
}
