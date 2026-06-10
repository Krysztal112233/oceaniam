use crate::support::spawn_app_with_isolated_schema;

/// GET /administrators/me — fetch current admin profile
/// Requires an admin JWT obtained via root_signin.
/// Asserts: returns 200 with a non-empty `id` and `name` = "root".
// NOTE: AI-generated test
#[tokio::test]
async fn get_administrator_self_returns_200() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;

    let resp: serde_json::Value = app
        .client
        .get(app.url("/administrators/me"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("get administrator self request failed")
        .json()
        .await
        .expect("get administrator self response parse failed");

    let id = resp["id"].as_str().expect("admin id should be present");
    assert!(!id.is_empty(), "admin id should not be empty");
    assert_eq!(
        resp["name"].as_str(),
        Some("root"),
        "admin name should be root"
    );
}
