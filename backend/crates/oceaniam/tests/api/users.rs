use oceaniam_common::sqid::Sqid;
use uuid::Uuid;

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

/// Tests `DELETE /tenants/{tenant_id}/applications/{application_id}/users/{user_id}`.
///
/// Creates a user, deletes it, and asserts the HTTP response is a success status code
/// and the user is no longer accessible (GET returns 404).
// NOTE: AI-generated test
#[tokio::test]
async fn create_and_delete_application_user() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap();
    let app_resp = app.api_create_application(&token, tenant_id).await;
    let application_id = app_resp["application_id"].as_str().unwrap().to_string();

    let user = app
        .api_create_user(&token, tenant_id, &application_id)
        .await;
    let user_id = user["id"].as_str().unwrap().to_string();

    let delete_resp = app
        .api_delete_application_user(&token, tenant_id, &application_id, &user_id)
        .await;
    assert!(
        delete_resp.status().is_success(),
        "delete application user should return success (got {})",
        delete_resp.status()
    );

    let get_resp = app
        .client
        .get(app.url(&format!(
            "/tenants/{tenant_id}/applications/{application_id}/users/{user_id}"
        )))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("get deleted user request failed");
    assert_eq!(
        get_resp.status(),
        404,
        "deleted application user should return 404"
    );
}

/// Tests `DELETE /tenants/{tenant_id}/applications/{application_id}/users/{user_id}`.
///
/// Asserts: deleting a user that does not exist returns 404.
// NOTE: AI-generated test
#[tokio::test]
async fn delete_nonexistent_application_user_returns_404() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap();
    let app_resp = app.api_create_application(&token, tenant_id).await;
    let application_id = app_resp["application_id"].as_str().unwrap().to_string();

    // Use a valid-looking but non-existent user ID (Sqid-encoded random UUID).
    let fake_user_id = Uuid::now_v7();
    let fake_user_sqid: Sqid = fake_user_id.into();

    let delete_resp = app
        .api_delete_application_user(&token, tenant_id, &application_id, fake_user_sqid.as_ref())
        .await;
    assert_eq!(
        delete_resp.status(),
        404,
        "deleting a nonexistent application user should return 404 (got {})",
        delete_resp.status()
    );
}

/// Tests `PATCH /tenants/{tenant_id}/applications/{application_id}/users/{user_id}`.
///
/// Creates a user, patches nickname, and asserts the response and subsequent GET
/// both reflect the new nickname while email/phone remain unchanged.
// NOTE: AI-generated test
#[tokio::test]
async fn patch_application_user_nickname_returns_updated_user() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap();
    let app_resp = app.api_create_application(&token, tenant_id).await;
    let application_id = app_resp["application_id"].as_str().unwrap().to_string();

    let user = app
        .api_create_user(&token, tenant_id, &application_id)
        .await;
    let user_id = user["id"].as_str().unwrap().to_string();
    let original_email = user["email"].as_str().map(str::to_owned);

    let patch_resp = app
        .client
        .patch(app.url(&format!(
            "/tenants/{tenant_id}/applications/{application_id}/users/{user_id}"
        )))
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!({ "nickname": "new_nickname" }))
        .send()
        .await
        .expect("patch user request failed");

    assert!(
        patch_resp.status().is_success(),
        "patch application user should return success (got {})",
        patch_resp.status()
    );

    let patched: serde_json::Value = patch_resp
        .json()
        .await
        .expect("patch user response parse failed");
    assert_eq!(
        patched["id"].as_str(),
        Some(user_id.as_str()),
        "patched user id should match"
    );
    assert_eq!(
        patched["nickname"].as_str(),
        Some("new_nickname"),
        "patched nickname should match request"
    );
    assert_eq!(
        patched["email"].as_str().map(str::to_owned),
        original_email,
        "email should remain unchanged"
    );

    let get_resp: serde_json::Value = app
        .client
        .get(app.url(&format!(
            "/tenants/{tenant_id}/applications/{application_id}/users/{user_id}"
        )))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("get user request failed")
        .json()
        .await
        .expect("get user response parse failed");

    assert_eq!(
        get_resp["nickname"].as_str(),
        Some("new_nickname"),
        "GET after patch should return updated nickname"
    );
}

/// Tests `PATCH /tenants/{tenant_id}/applications/{application_id}/users/{user_id}`.
///
/// Asserts: patching a user that does not exist returns 404.
// NOTE: AI-generated test
#[tokio::test]
async fn patch_nonexistent_application_user_returns_404() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().unwrap();
    let app_resp = app.api_create_application(&token, tenant_id).await;
    let application_id = app_resp["application_id"].as_str().unwrap().to_string();

    let fake_user_id = Uuid::now_v7();
    let fake_user_sqid: Sqid = fake_user_id.into();

    let patch_resp = app
        .client
        .patch(app.url(&format!(
            "/tenants/{tenant_id}/applications/{application_id}/users/{fake_user_sqid}"
        )))
        .header("Authorization", format!("Bearer {token}"))
        .json(&serde_json::json!({ "nickname": "new_nickname" }))
        .send()
        .await
        .expect("patch nonexistent user request failed");

    assert_eq!(
        patch_resp.status(),
        404,
        "patching a nonexistent application user should return 404 (got {})",
        patch_resp.status()
    );
}
