//! OpenID Provider Configuration (discovery) endpoint tests.

use oceaniam_common::consts::SYSTEM_TENANT_UUID;
use oceaniam_common::sqid::Sqid;
use uuid::Uuid;

use crate::support::{
    oidc_drive::{drive_user_token_response, seed_login_fixture, spawn_preview},
    spawn_app_with_isolated_schema,
};

/// Append-style path (OIDC Discovery 1.0 §4.1).
fn append_path(tenant_sqid: &str) -> String {
    format!("/oidc/{tenant_sqid}/.well-known/openid-configuration")
}

/// Insert-style path (RFC 8414 §3.1).
fn insert_path(tenant_sqid: &str) -> String {
    format!("/.well-known/openid-configuration/oidc/{tenant_sqid}")
}

/// The tenant issuer under the hard-coded test public base URL
/// (`OCEANIAM_PUBLIC_BASE_URL=http://localhost:8000` in `tests/support/mod.rs`).
fn tenant_issuer(tenant_sqid: &str) -> String {
    format!("http://localhost:8000/oidc/{tenant_sqid}")
}

fn expected_configuration(tenant_sqid: &str) -> serde_json::Value {
    let issuer = tenant_issuer(tenant_sqid);
    serde_json::json!({
        "issuer": issuer,
        "authorization_endpoint": format!("{issuer}/authorize"),
        "token_endpoint": format!("{issuer}/token"),
        "userinfo_endpoint": format!("{issuer}/userinfo"),
        "jwks_uri": format!("{issuer}/jwks.json"),
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["PS512"],
        "scopes_supported": ["openid"],
        "token_endpoint_auth_methods_supported": ["none"],
        "code_challenge_methods_supported": ["S256"],
        "request_uri_parameter_supported": false,
        "claims_supported": ["iss", "sub", "aud", "exp", "iat", "auth_time", "nonce"],
    })
}

/// Both well-known path shapes serve a byte-identical document whose every advertised field
/// matches the implemented capability exactly, with no fields beyond the advertised set.
// NOTE: AI-generated test
#[tokio::test]
async fn discovery_document_is_byte_identical_on_both_paths_with_exact_fields() {
    let app = spawn_preview().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_sqid = tenant["id"].as_str().unwrap().to_string();

    let append_resp = app
        .client
        .get(app.url(&append_path(&tenant_sqid)))
        .send()
        .await
        .expect("append-style discovery request failed");
    assert_eq!(append_resp.status(), 200, "append-style path returns 200");
    assert!(
        append_resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| v.starts_with("application/json")),
        "discovery returns a JSON content type"
    );

    let insert_resp = app
        .client
        .get(app.url(&insert_path(&tenant_sqid)))
        .send()
        .await
        .expect("insert-style discovery request failed");
    assert_eq!(insert_resp.status(), 200, "insert-style path returns 200");

    let append_body = append_resp.text().await.expect("append body");
    let insert_body = insert_resp.text().await.expect("insert body");
    assert_eq!(
        append_body, insert_body,
        "both well-known path shapes must serve a byte-identical document"
    );

    let body: serde_json::Value =
        serde_json::from_str(&append_body).expect("discovery body should be JSON");
    assert_eq!(
        body,
        expected_configuration(&tenant_sqid),
        "the whole document must equal the expected capability mirror, with no extra fields"
    );
}

/// `Cache-Control: public, max-age=60` on both path shapes, mirroring the tenant JWKS.
// NOTE: AI-generated test
#[tokio::test]
async fn discovery_document_carries_the_bounded_public_cache_header_on_both_paths() {
    let app = spawn_preview().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_sqid = tenant["id"].as_str().unwrap().to_string();

    for path in [append_path(&tenant_sqid), insert_path(&tenant_sqid)] {
        let response = app
            .client
            .get(app.url(&path))
            .send()
            .await
            .expect("discovery request failed");
        assert_eq!(response.status(), 200, "path {path}");
        assert_eq!(
            response
                .headers()
                .get("cache-control")
                .and_then(|v| v.to_str().ok()),
            Some("public, max-age=60"),
            "path {path} should carry the bounded public cache header"
        );
    }
}

/// Tenant-namespace semantics mirror the JWKS route on both path shapes: a malformed tenant
/// identifier is 400, an absent tenant is 404, and the system tenant is 404.
///
/// The malformed case extracts through serde (`Sqid` is a Deserialize newtype; axum `Path`
/// never calls `FromStr`) and fails at the Sqid→Uuid conversion, so the 400 is the JSON
/// `ApiResponse<ErrorResponse>` shape the OpenAPI declaration documents — pinned here in both
/// directions (status and content type) so the declaration cannot drift again.
// NOTE: AI-generated test
#[tokio::test]
async fn malformed_unknown_and_system_tenants_are_rejected() {
    let app = spawn_preview().await;
    let unknown_sqid = Sqid::from(Uuid::now_v7()).to_string();
    let system_sqid = Sqid::from(SYSTEM_TENANT_UUID).to_string();

    for (label, tenant, status) in [
        ("malformed", "@@not-a-sqid@@".to_string(), 400),
        ("unknown", unknown_sqid, 404),
        ("system", system_sqid, 404),
    ] {
        for path in [append_path(&tenant), insert_path(&tenant)] {
            let response = app
                .client
                .get(app.url(&path))
                .send()
                .await
                .expect("discovery request failed");
            assert_eq!(response.status(), status, "{label} tenant on path {path}");

            if label == "malformed" {
                assert_eq!(
                    response
                        .headers()
                        .get("content-type")
                        .and_then(|value| value.to_str().ok()),
                    Some("application/json"),
                    "malformed tenant on path {path} returns the JSON error shape"
                );
                let body: serde_json::Value = response.json().await.expect("400 JSON body");
                assert_eq!(
                    body["msg"], "cannot parse input id",
                    "malformed tenant on path {path} carries the invalid-sqid message"
                );
                assert!(
                    body["error_id"].as_str().is_some_and(|id| !id.is_empty()),
                    "malformed tenant on path {path} carries an error id"
                );
            }
        }
    }
}

/// Discovery is gated by the same default-disabled authorization-entry preview flag as the
/// protocol family it describes: with the preview disabled, both well-known paths answer the
/// gate's plain-text 404 before any tenant resolution.
// NOTE: AI-generated test
#[tokio::test]
async fn disabled_preview_gate_is_404_on_both_paths() {
    let app = spawn_app_with_isolated_schema().await;
    // The management API is not preview-gated, so a REAL tenant can be seeded on a
    // preview-disabled app.
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_sqid = tenant["id"].as_str().unwrap().to_string();

    for path in [append_path(&tenant_sqid), insert_path(&tenant_sqid)] {
        let response = app
            .client
            .get(app.url(&path))
            .send()
            .await
            .expect("discovery request failed");
        assert_eq!(response.status(), 404, "path {path}");
        assert_eq!(
            response
                .headers()
                .get("content-type")
                .and_then(|value| value.to_str().ok()),
            Some("text/plain; charset=utf-8")
        );
        let body = response.text().await.expect("read gate response body");
        assert_eq!(body, "not found");
    }
}

/// Behavior-level pin for `id_token_signing_alg_values_supported`: drive a real login and
/// token exchange, then verify the issued ID token's `alg` header is advertised by the
/// discovery document (and is PS512, the tenant keybox generation algorithm).
// NOTE: AI-generated test
#[tokio::test]
async fn issued_id_token_alg_is_advertised_by_the_discovery_document() {
    let app = spawn_preview().await;
    let fixture = seed_login_fixture(&app).await;
    let (_, token_body) = drive_user_token_response(
        &app,
        &fixture,
        "discovery-alg@example.com",
        "discovery-alg-state",
    )
    .await;
    let id_token = token_body["id_token"]
        .as_str()
        .expect("token response should carry an id_token");

    let header = oceaniam_auth::decode_header(id_token).expect("id_token header should decode");
    let issued_alg = serde_json::to_value(header.alg).expect("alg serializes");

    let response = app
        .client
        .get(app.url(&append_path(&fixture.tenant_sqid)))
        .send()
        .await
        .expect("discovery request failed");
    assert_eq!(response.status(), 200);
    let discovery: serde_json::Value = response.json().await.expect("discovery body");
    let advertised = discovery["id_token_signing_alg_values_supported"]
        .as_array()
        .expect("advertised alg list");

    assert_eq!(
        issued_alg, "PS512",
        "tenant keyboxes sign ID tokens with PS512"
    );
    assert!(
        advertised.contains(&issued_alg),
        "the discovery document must advertise the alg of a genuinely issued ID token: \
         issued {issued_alg}, advertised {advertised:?}"
    );
}

/// The generated OpenAPI document registers both well-known path shapes.
// NOTE: AI-generated test
#[test]
fn openapi_spec_contains_both_discovery_paths() {
    let spec = oceaniam::app::build_openapi_spec();
    assert!(
        spec.paths
            .paths
            .contains_key("/oidc/{tenant_sqid}/.well-known/openid-configuration"),
        "append-style discovery path must be registered in the OpenAPI document"
    );
    assert!(
        spec.paths
            .paths
            .contains_key("/.well-known/openid-configuration/oidc/{tenant_sqid}"),
        "insert-style discovery path must be registered in the OpenAPI document"
    );
}
