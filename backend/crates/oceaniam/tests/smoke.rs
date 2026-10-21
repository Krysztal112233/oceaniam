// Rule 1 test exemption (`oceaniam-database/src/helper/README.md`): seeding and asserting
// state with SeaORM builders is the legitimate job of tests.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

mod support;

use crate::support::spawn_app_with_isolated_schema;

// NOTE: AI-generated test
#[tokio::test]
async fn root_endpoint_returns_ok() {
    let app = spawn_app_with_isolated_schema().await;

    let response = app
        .client
        .get(app.url("/"))
        .send()
        .await
        .expect("failed to send request to integration test server");

    assert!(response.status().is_success());
}
