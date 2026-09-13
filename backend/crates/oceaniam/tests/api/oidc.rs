use oceaniam_common::consts::SYSTEM_TENANT_UUID;
use oceaniam_common::sqid::Sqid;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter};
use uuid::Uuid;

use crate::support::spawn_app_with_isolated_schema;

mod authorization;
mod login;

fn public_jwk_object_is_clean(key: &serde_json::Value) {
    let object = key.as_object().expect("each JWK should be an object");
    let public_fields = ["kty", "kid", "use", "alg", "n", "e"];

    assert!(
        object
            .keys()
            .all(|field| public_fields.contains(&field.as_str())),
        "JWK must contain only public RSA verification fields: {object:?}"
    );
    for private_field in ["d", "p", "q", "dp", "dq", "qi", "oth", "k"] {
        assert!(
            object.get(private_field).is_none(),
            "private or symmetric field `{private_field}` must not be exposed"
        );
    }
}

/// GET /oidc/{tenant_sqid}/jwks.json
///
/// A real tenant returns 200 with a non-empty public-only JWK Set, correct headers, and a body
/// identical to the legacy tenant JWKS route.
// NOTE: AI-generated test
#[tokio::test]
async fn oidc_jwks_serves_public_keys_with_parity_and_headers() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant = app.api_create_tenant(&token).await;
    let tenant_sqid = tenant["id"].as_str().unwrap().to_string();

    let oidc_resp = app
        .client
        .get(app.url(&format!("/oidc/{tenant_sqid}/jwks.json")))
        .send()
        .await
        .expect("oidc jwks request failed");
    assert_eq!(oidc_resp.status(), 200, "oidc jwks should return 200");
    assert_eq!(
        oidc_resp
            .headers()
            .get("cache-control")
            .and_then(|v| v.to_str().ok()),
        Some("public, max-age=60"),
        "oidc jwks should carry the bounded public cache header"
    );
    assert!(
        oidc_resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|v| v.starts_with("application/json")),
        "oidc jwks should return JSON content type"
    );

    let oidc_body: serde_json::Value = oidc_resp
        .json()
        .await
        .expect("oidc jwks body should be JSON");

    let keys = oidc_body["keys"]
        .as_array()
        .expect("keys should be an array");
    assert!(
        !keys.is_empty(),
        "oidc jwks should contain at least one key"
    );
    for key in keys {
        assert_eq!(key["kty"], "RSA", "tenant keys are RSA");
        public_jwk_object_is_clean(key);
    }

    let legacy_body: serde_json::Value = app
        .client
        .get(app.url(&format!("/tenants/{tenant_sqid}/.well-known/jwks.json")))
        .send()
        .await
        .expect("legacy jwks request failed")
        .json()
        .await
        .expect("legacy jwks body should be JSON");

    assert_eq!(
        oidc_body["keys"], legacy_body["keys"],
        "both routes must expose the same tenant JWK Set"
    );
}

/// GET /oidc/{invalid}/jwks.json, unknown tenant on both routes
///
/// An invalid Sqid is rejected with 400; a valid Sqid for an absent tenant returns 404 on the
/// OIDC route and on the legacy route, and no `key_boxes` rows are ever written for it.
// NOTE: AI-generated test
#[tokio::test]
async fn unknown_and_invalid_tenants_are_rejected_without_key_writes() {
    let app = spawn_app_with_isolated_schema().await;
    let database = app.database().await;
    let unknown_tenant_id = Uuid::now_v7();
    let unknown_sqid = Sqid::from(unknown_tenant_id).to_string();

    let oidc_unknown = app
        .client
        .get(app.url(&format!("/oidc/{unknown_sqid}/jwks.json")))
        .send()
        .await
        .expect("oidc jwks request failed");
    assert_eq!(oidc_unknown.status(), 404, "absent tenant should be 404");

    let legacy_unknown = app
        .client
        .get(app.url(&format!("/tenants/{unknown_sqid}/.well-known/jwks.json")))
        .send()
        .await
        .expect("legacy jwks request failed");
    assert_eq!(
        legacy_unknown.status(),
        404,
        "legacy route must not generate keys for absent tenants"
    );

    let invalid_resp = app
        .client
        .get(app.url("/oidc/@@not-a-sqid@@/jwks.json"))
        .send()
        .await
        .expect("invalid sqid request failed");
    assert_eq!(invalid_resp.status(), 400, "invalid sqid should be 400");

    assert_eq!(
        count_tenant_keys(&database, unknown_tenant_id).await,
        0,
        "no key material may be generated for absent tenants"
    );
}

/// GET /oidc/{system_sqid}/jwks.json vs legacy system route
///
/// The system tenant is excluded from the OIDC namespace (404) while the legacy system JWKS
/// route keeps succeeding.
// NOTE: AI-generated test
#[tokio::test]
async fn system_tenant_is_excluded_from_oidc_but_legacy_route_still_works() {
    let app = spawn_app_with_isolated_schema().await;
    let system_sqid = Sqid::from(SYSTEM_TENANT_UUID).to_string();

    let oidc_resp = app
        .client
        .get(app.url(&format!("/oidc/{system_sqid}/jwks.json")))
        .send()
        .await
        .expect("oidc jwks request failed");
    assert_eq!(
        oidc_resp.status(),
        404,
        "system tenant is not an OIDC issuer"
    );

    let legacy_resp = app
        .client
        .get(app.url(&format!("/tenants/{system_sqid}/.well-known/jwks.json")))
        .send()
        .await
        .expect("legacy system jwks request failed");
    assert_eq!(
        legacy_resp.status(),
        200,
        "legacy system JWKS route must keep working"
    );
    let body: serde_json::Value = legacy_resp.json().await.expect("system jwks body");
    assert!(
        !body["keys"].as_array().expect("keys array").is_empty(),
        "system tenant must keep its signing keys"
    );
}

/// GET /oidc/{tenant_sqid}/jwks.json for two different tenants
///
/// Distinct tenants never observe each other's keys.
// NOTE: AI-generated test
#[tokio::test]
async fn oidc_jwks_isolates_tenants() {
    let app = spawn_app_with_isolated_schema().await;
    let token = app.root_signin().await;
    let tenant_a = app.api_create_tenant(&token).await;
    let tenant_b = app.api_create_tenant(&token).await;
    let sqid_a = tenant_a["id"].as_str().unwrap().to_string();
    let sqid_b = tenant_b["id"].as_str().unwrap().to_string();

    let body_a: serde_json::Value = app
        .client
        .get(app.url(&format!("/oidc/{sqid_a}/jwks.json")))
        .send()
        .await
        .expect("oidc jwks request failed")
        .json()
        .await
        .expect("oidc jwks body should be JSON");
    let body_b: serde_json::Value = app
        .client
        .get(app.url(&format!("/oidc/{sqid_b}/jwks.json")))
        .send()
        .await
        .expect("oidc jwks request failed")
        .json()
        .await
        .expect("oidc jwks body should be JSON");

    let kids_a = collect_kids(&body_a);
    let kids_b = collect_kids(&body_b);

    assert!(
        !kids_a.is_empty() && !kids_b.is_empty(),
        "both tenants have keys"
    );
    assert!(
        kids_a.iter().all(|kid| !kids_b.contains(kid)),
        "tenant key ids must not overlap"
    );
}

/// The generated OpenAPI document registers the OIDC JWKS route.
// NOTE: AI-generated test
#[test]
fn openapi_spec_contains_oidc_jwks_path() {
    let spec = oceaniam::app::build_openapi_spec();
    assert!(
        spec.paths
            .paths
            .contains_key("/oidc/{tenant_sqid}/jwks.json"),
        "OIDC JWKS path must be registered in the OpenAPI document"
    );
}

async fn count_tenant_keys(database: &DatabaseConnection, tenant_id: Uuid) -> u64 {
    oceaniam_database::model::prelude::KeyBoxes::find()
        .filter(oceaniam_database::model::key_boxes::Column::TenantId.eq(tenant_id))
        .count(database)
        .await
        .expect("counting tenant keys should not fail")
}

fn collect_kids(body: &serde_json::Value) -> Vec<String> {
    body["keys"]
        .as_array()
        .expect("keys should be an array")
        .iter()
        .map(|key| key["kid"].as_str().unwrap_or_default().to_string())
        .collect()
}
