mod support;

use crate::support::{spawn_app_with_isolated_schema, test_config};

// NOTE: AI-generated test
#[tokio::test]
async fn root_endpoint_returns_ok() {
    let app = spawn_app_with_isolated_schema(test_config()).await;

    let response = app
        .client
        .get(app.url("/"))
        .send()
        .await
        .expect("failed to send request to integration test server");

    assert!(response.status().is_success());
}
