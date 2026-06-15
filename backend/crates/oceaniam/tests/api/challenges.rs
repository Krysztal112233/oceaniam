use std::str::FromStr;

use oceaniam_common::sqid::Sqid;
use uuid::Uuid;

use crate::support::spawn_app_with_isolated_schema;

// NOTE: AI-generated test
#[tokio::test]
async fn create_email_totp_challenge_then_verify_returns_jwt() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_id = tenant["id"].as_str().expect("tenant id should be present");
    let app_resp = app.api_create_application(&token, tenant_id).await;
    let application_id = app_resp["application_id"]
        .as_str()
        .expect("application id should be present");
    let user = app.api_create_user(&token, tenant_id, application_id).await;
    let subject_id = user["id"].as_str().expect("user id should be present");
    let subject_uuid: Uuid = Sqid::from_str(subject_id)
        .expect("user id should be a Sqid")
        .try_into()
        .expect("user id should decode to a UUID");
    let subject_uuid_string = subject_uuid.to_string();
    let body = serde_json::json!({
        "subject_id": subject_uuid,
        "factor_type": "email_totp",
        "payload": {
            "code": "123456"
        }
    });

    let response = app
        .client
        .post(app.url(&format!(
            "/tenants/{tenant_id}/applications/{application_id}/challenges"
        )))
        .header("Authorization", format!("Bearer {token}"))
        .json(&body)
        .send()
        .await
        .expect("create challenge request failed");
    let status = response.status();
    let body = response.text().await.expect("response body should be read");

    assert!(
        status.is_success(),
        "create challenge should succeed, got status={status}, body={body}"
    );

    let challenge: serde_json::Value =
        serde_json::from_str(&body).expect("create challenge response parse failed");

    assert_eq!(challenge["factor_type"].as_str(), Some("email_totp"));
    assert_eq!(
        challenge["subject_id"].as_str(),
        Some(subject_uuid_string.as_str())
    );

    let challenge_id = challenge["id"]
        .as_str()
        .expect("challenge id should be present");
    let verify_body = serde_json::json!({ "code": "123456" });
    let response = app
        .client
        .post(app.url(&format!(
            "/tenants/{tenant_id}/applications/{application_id}/challenges/{challenge_id}"
        )))
        .json(&verify_body)
        .send()
        .await
        .expect("verify challenge request failed");
    let status = response.status();
    let body = response.text().await.expect("response body should be read");

    assert!(
        status.is_success(),
        "verify challenge should succeed, got status={status}, body={body}"
    );

    let signin: serde_json::Value =
        serde_json::from_str(&body).expect("verify challenge response parse failed");
    let jwt = signin["jwt"].as_str().expect("jwt should be present");
    assert!(!jwt.is_empty(), "jwt should not be empty");
}
