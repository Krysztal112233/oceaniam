use std::collections::HashMap;

use axum_extra::extract::cookie::{Cookie, SameSite};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use oceaniam_common::{config::BackendConfig, consts::SYSTEM_TENANT_UUID, sqid::Sqid};
use oceaniam_database::{
    config::application::ApplicationConfiguration,
    helper::{applications::ApplicationHelper, oidc_clients::OidcClientsHelper},
    model::{
        oidc_clients,
        prelude::{Applications, OidcAuthorizationTransactions},
        sea_orm_active_enums::{OidcAuthorizationTransactionStatus, OidcPkceMethod},
    },
};
use oceaniam_oidc::{authorization_browser_binding_digest, authorization_csrf_digest};
use reqwest::{Client, Response, redirect::Policy};
use sea_orm::{ConnectionTrait, EntityTrait, PaginatorTrait};
use time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use url::{Url, form_urlencoded};
use uuid::Uuid;

use crate::support::{
    TestApp, spawn_app_with_isolated_schema, spawn_app_with_isolated_schema_configured,
};

const CODE_CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
const AUTHORIZATION_HTTP_LIMIT_BYTES: usize = 8 * 1024;

struct AuthorizationFixture {
    tenant_id: Uuid,
    tenant_sqid: String,
    application_id: Uuid,
    oidc_client_id: Uuid,
    client_id: String,
    redirect_uri: String,
}

async fn spawn_preview(public_base_url: &str) -> TestApp {
    let public_base_url = public_base_url.to_owned();
    spawn_app_with_isolated_schema_configured(move |config: &mut BackendConfig| {
        config.oidc.authorization_entry_preview_enabled = true;
        config.public_base_url = Url::parse(&public_base_url)
            .expect("test public base URL")
            .try_into()
            .expect("valid test public base URL");
    })
    .await
}

async fn seed_authorization_client(
    app: &TestApp,
    redirect_uri: &str,
    allow_insecure_loopback: bool,
) -> AuthorizationFixture {
    let database = app.database().await;
    let (tenant_id, application_id) = app.seed_tenant_and_application().await;
    if allow_insecure_loopback {
        let mut configuration = ApplicationConfiguration::default();
        configuration.oidc.allow_insecure_loopback_redirect_uris = true;
        Applications::replace_configuration(application_id, configuration, &database)
            .await
            .expect("enable fixture-specific loopback redirect policy");
    }

    let oidc_client_id = Uuid::now_v7();
    let client_id = format!("preview-client-{}", oidc_client_id.simple());
    oidc_clients::Entity::create_client(
        oidc_client_id,
        application_id,
        client_id.clone(),
        "Authorization preview test client".to_owned(),
        &database,
    )
    .await
    .expect("create authorization preview client");
    oidc_clients::Entity::create_redirect_uris(
        oidc_client_id,
        vec![redirect_uri.to_owned()],
        &database,
    )
    .await
    .expect("create authorization preview redirect");

    AuthorizationFixture {
        tenant_id,
        tenant_sqid: Sqid::from(tenant_id).to_string(),
        application_id,
        oidc_client_id,
        client_id,
        redirect_uri: redirect_uri.to_owned(),
    }
}

fn authorization_form(
    fixture: &AuthorizationFixture,
    state: &str,
    response_type: &str,
    extras: &[(&str, &str)],
) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    for (name, value) in [
        ("response_type", response_type),
        ("client_id", fixture.client_id.as_str()),
        ("redirect_uri", fixture.redirect_uri.as_str()),
        ("scope", "openid"),
        ("state", state),
        ("nonce", "private-nonce-sentinel"),
        ("code_challenge", CODE_CHALLENGE),
        ("code_challenge_method", "S256"),
    ] {
        serializer.append_pair(name, value);
    }
    for (name, value) in extras {
        serializer.append_pair(name, value);
    }
    serializer.finish()
}

fn pad_authorization_form_to_len(mut encoded: String, exact_len: usize) -> String {
    const PADDING_PREFIX: &str = "&ignored_boundary_padding=";

    let padding_len = exact_len
        .checked_sub(encoded.len() + PADDING_PREFIX.len())
        .expect("valid authorization form should fit below the requested boundary");
    encoded.push_str(PADDING_PREFIX);
    encoded.push_str(&"x".repeat(padding_len));
    assert_eq!(encoded.len(), exact_len);
    encoded
}

fn authorize_path(fixture: &AuthorizationFixture) -> String {
    format!("/oidc/{}/authorize", fixture.tenant_sqid)
}

fn no_redirect_client() -> Client {
    Client::builder()
        .redirect(Policy::none())
        .build()
        .expect("build no-redirect client")
}

fn assert_common_headers(response: &Response) {
    assert_eq!(
        response
            .headers()
            .get("cache-control")
            .and_then(|value| value.to_str().ok()),
        Some("no-store")
    );
    assert_eq!(
        response
            .headers()
            .get("pragma")
            .and_then(|value| value.to_str().ok()),
        Some("no-cache")
    );
    assert_eq!(
        response
            .headers()
            .get("referrer-policy")
            .and_then(|value| value.to_str().ok()),
        Some("no-referrer")
    );
    assert_eq!(
        response
            .headers()
            .get("x-content-type-options")
            .and_then(|value| value.to_str().ok()),
        Some("nosniff")
    );
}

fn html_attribute(body: &str, name: &str) -> String {
    let marker = format!("{name}=\"");
    let start = body.find(&marker).expect("HTML attribute should exist") + marker.len();
    let end = body[start..]
        .find('"')
        .map(|offset| start + offset)
        .expect("HTML attribute should close");
    body[start..end].to_owned()
}

fn decode_secret(value: &str) -> [u8; 32] {
    URL_SAFE_NO_PAD
        .decode(value)
        .expect("secret should use canonical base64url")
        .try_into()
        .expect("secret should contain 32 bytes")
}

async fn assert_success_snapshot(
    app: &TestApp,
    fixture: &AuthorizationFixture,
    response: Response,
    expected_state: &str,
    expect_secure_cookie: bool,
) -> Uuid {
    assert_eq!(response.status(), 200);
    assert_common_headers(&response);
    assert!(
        response
            .headers()
            .get("content-security-policy")
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| {
                value.contains("script-src 'none'")
                    && value.contains("form-action 'none'")
                    && value.contains("frame-ancestors 'none'")
            })
    );
    let cookie_header = response
        .headers()
        .get("set-cookie")
        .expect("success should set one binding cookie")
        .to_str()
        .expect("binding cookie should be ASCII")
        .to_owned();
    let body = response.text().await.expect("read static HTML");

    assert!(body.contains("Sign-in is not available"));
    assert!(body.contains("hidden"));
    assert!(!body.contains("<form"));
    assert!(!body.contains("password"));
    assert!(!body.contains(expected_state));
    assert!(!body.contains("private-nonce-sentinel"));
    assert!(!body.contains(&fixture.redirect_uri));

    let transaction_sqid = html_attribute(&body, "data-transaction-sqid");
    let transaction_id = Uuid::try_from(
        transaction_sqid
            .parse::<Sqid>()
            .expect("transaction context should be a Sqid"),
    )
    .expect("transaction Sqid should decode");
    let csrf = decode_secret(&html_attribute(&body, "data-csrf"));
    assert_eq!(html_attribute(&body, "data-revision"), "0");

    let cookie = Cookie::parse(cookie_header)
        .expect("binding cookie should parse")
        .into_owned();
    assert_eq!(
        cookie.name(),
        format!("oceaniam_oidc_binding_{transaction_sqid}")
    );
    assert_eq!(cookie.http_only(), Some(true));
    assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    assert_eq!(cookie.secure(), expect_secure_cookie.then_some(true));
    assert_eq!(
        cookie.domain(),
        None,
        "binding cookie must remain host-only"
    );
    assert_eq!(
        cookie.path(),
        Some(format!("/oidc/{}", fixture.tenant_sqid).as_str())
    );
    assert_eq!(cookie.max_age(), Some(Duration::seconds(600)));
    let browser_binding = decode_secret(cookie.value());

    let database = app.database().await;
    let snapshot = OidcAuthorizationTransactions::find_by_id(transaction_id)
        .one(&database)
        .await
        .expect("read authorization snapshot")
        .expect("successful response must have a committed snapshot");
    assert_eq!(snapshot.tenant_id, fixture.tenant_id);
    assert_eq!(snapshot.application_id, fixture.application_id);
    assert_eq!(snapshot.oidc_client_id, fixture.oidc_client_id);
    assert_eq!(snapshot.redirect_uri, fixture.redirect_uri);
    assert_eq!(snapshot.state, expected_state);
    assert_eq!(snapshot.requested_scope, "openid");
    assert_eq!(snapshot.nonce.as_deref(), Some("private-nonce-sentinel"));
    assert_eq!(snapshot.code_challenge, CODE_CHALLENGE);
    assert_eq!(snapshot.code_challenge_method, OidcPkceMethod::S256);
    assert_eq!(snapshot.status, OidcAuthorizationTransactionStatus::Pending);
    assert_eq!(snapshot.revision, 0);
    assert!(snapshot.terminal_at.is_none());
    assert_eq!(
        snapshot.browser_binding_digest,
        authorization_browser_binding_digest(&browser_binding)
    );
    assert_eq!(snapshot.csrf_digest, authorization_csrf_digest(&csrf));
    assert_ne!(snapshot.browser_binding_digest, snapshot.csrf_digest);
    assert_eq!(
        snapshot.expires_at - snapshot.created_at,
        chrono::Duration::minutes(10)
    );

    transaction_id
}

async fn assert_rejected_without_success_context(_app: &TestApp, response: Response, status: u16) {
    assert_eq!(response.status().as_u16(), status);
    assert_common_headers(&response);
    assert!(response.headers().get("set-cookie").is_none());
}

async fn send_chunked_oversized_form(app: &TestApp, path: &str) -> String {
    let origin = Url::parse(&app.address).expect("test app address should be a URL");
    let host = origin.host_str().expect("test app should have a host");
    let port = origin.port().expect("test app should have a port");
    let mut connection = tokio::net::TcpStream::connect((host, port))
        .await
        .expect("connect raw chunked request");
    let headers = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/x-www-form-urlencoded\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n"
    );
    connection
        .write_all(headers.as_bytes())
        .await
        .expect("write chunked request headers");
    for chunk in [vec![b'x'; 4096], vec![b'y'; 4097]] {
        connection
            .write_all(format!("{:x}\r\n", chunk.len()).as_bytes())
            .await
            .expect("write chunk length");
        connection
            .write_all(&chunk)
            .await
            .expect("write chunk data");
        connection
            .write_all(b"\r\n")
            .await
            .expect("terminate chunk");
    }
    connection
        .write_all(b"0\r\n\r\n")
        .await
        .expect("finish chunked body");

    let mut response = Vec::new();
    connection
        .read_to_end(&mut response)
        .await
        .expect("read raw chunked response");
    String::from_utf8(response).expect("HTTP response should be ASCII/UTF-8")
}

// NOTE: AI-generated test
#[tokio::test]
async fn get_and_form_post_create_independent_bound_snapshots_and_static_html() {
    let app = spawn_preview("https://issuer.example").await;
    let fixture = seed_authorization_client(&app, "https://client.example/callback", false).await;

    let get_state = "private-state-get-sentinel";
    let get_form = authorization_form(&fixture, get_state, "code", &[]);
    let get_response = app
        .client
        .get(app.url(&format!("{}?{get_form}", authorize_path(&fixture))))
        .header("host", "attacker.example")
        .header("forwarded", "host=attacker.example;proto=http")
        .send()
        .await
        .expect("GET authorization preview");
    let get_id = assert_success_snapshot(&app, &fixture, get_response, get_state, true).await;

    let post_state = "private-state-post-sentinel";
    let post_form = authorization_form(&fixture, post_state, "code", &[]);
    let post_response = app
        .client
        .post(app.url(&authorize_path(&fixture)))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(post_form)
        .send()
        .await
        .expect("POST authorization preview");
    let post_id = assert_success_snapshot(&app, &fixture, post_response, post_state, true).await;

    assert_ne!(get_id, post_id, "requests must never deduplicate snapshots");
    let database = app.database().await;
    let snapshots = OidcAuthorizationTransactions::find()
        .all(&database)
        .await
        .expect("list authorization snapshots");
    assert_eq!(snapshots.len(), 2);
    assert!(
        snapshots.iter().all(|snapshot| snapshot.issuer
            == format!("https://issuer.example/oidc/{}", fixture.tenant_sqid)),
        "issuer must use trusted configuration and canonical tenant Sqid"
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn preview_gate_precedes_body_consumption_and_methods_remain_non_creating() {
    let disabled = spawn_app_with_isolated_schema().await;
    let tenant_sqid = Sqid::from(Uuid::now_v7()).to_string();
    let disabled_response = disabled
        .client
        .post(disabled.url(&format!("/oidc/{tenant_sqid}/authorize")))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(vec![b'x'; 9 * 1024])
        .send()
        .await
        .expect("disabled preview request");
    assert_rejected_without_success_context(&disabled, disabled_response, 404).await;
    let disabled_database = disabled.database().await;
    assert_eq!(
        OidcAuthorizationTransactions::find()
            .count(&disabled_database)
            .await
            .expect("count disabled snapshots"),
        0
    );

    let enabled = spawn_preview("http://localhost:8000").await;
    let fixture =
        seed_authorization_client(&enabled, "https://client.example/callback", false).await;
    let path = enabled.url(&authorize_path(&fixture));

    let head = enabled
        .client
        .head(&path)
        .send()
        .await
        .expect("HEAD authorization preview");
    assert_eq!(
        head.headers()
            .get("allow")
            .and_then(|value| value.to_str().ok()),
        Some("GET, POST")
    );
    assert_rejected_without_success_context(&enabled, head, 405).await;

    let put = enabled
        .client
        .put(&path)
        .send()
        .await
        .expect("unsupported method");
    assert_eq!(put.status(), 405);
    assert!(put.headers().get("set-cookie").is_none());

    let options = enabled
        .client
        .request(reqwest::Method::OPTIONS, &path)
        .header("origin", "http://localhost:8000")
        .header("access-control-request-method", "POST")
        .send()
        .await
        .expect("CORS preflight");
    assert!(options.status().is_success());
    assert_eq!(
        options
            .headers()
            .get("access-control-allow-origin")
            .and_then(|value| value.to_str().ok()),
        Some("http://localhost:8000")
    );

    let database = enabled.database().await;
    assert_eq!(
        OidcAuthorizationTransactions::find()
            .count(&database)
            .await
            .expect("count method snapshots"),
        0
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn transport_rejections_are_bounded_strict_and_never_persist() {
    let app = spawn_preview("http://localhost:8000").await;
    let fixture = seed_authorization_client(&app, "https://client.example/callback", false).await;
    let path = authorize_path(&fixture);

    let target_too_long = app
        .client
        .get(app.url(&format!("{path}?padding={}", "x".repeat(8 * 1024))))
        .send()
        .await
        .expect("oversized target request");
    assert_rejected_without_success_context(&app, target_too_long, 414).await;

    let body_too_long = app
        .client
        .post(app.url(&path))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(vec![b'x'; 8 * 1024 + 1])
        .send()
        .await
        .expect("oversized known-length body");
    assert_rejected_without_success_context(&app, body_too_long, 413).await;

    let chunked_too_long = send_chunked_oversized_form(&app, &path).await;
    let chunked_headers = chunked_too_long
        .split_once("\r\n\r\n")
        .map(|(headers, _)| headers)
        .expect("chunked response should contain HTTP headers")
        .to_ascii_lowercase();
    assert!(chunked_headers.starts_with("http/1.1 413 "));
    assert!(chunked_headers.contains("\r\ncache-control: no-store"));
    assert!(chunked_headers.contains("\r\npragma: no-cache"));
    assert!(chunked_headers.contains("\r\nreferrer-policy: no-referrer"));
    assert!(!chunked_headers.contains("\r\nset-cookie:"));

    let wrong_media = app
        .client
        .post(app.url(&path))
        .header("content-type", "application/json")
        .body("{}")
        .send()
        .await
        .expect("wrong media type");
    assert_rejected_without_success_context(&app, wrong_media, 415).await;

    let mixed_query = app
        .client
        .post(app.url(&format!("{path}?state=mixed")))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(authorization_form(&fixture, "body-state", "code", &[]))
        .send()
        .await
        .expect("mixed query request");
    assert_rejected_without_success_context(&app, mixed_query, 400).await;

    for malformed in [
        "unknown=%GG",
        "unknown=%ff",
        "unknown=%00",
        "client_id=first&%63lient_id=second",
    ] {
        let response = app
            .client
            .post(app.url(&path))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(malformed)
            .send()
            .await
            .expect("strict parser rejection");
        assert_rejected_without_success_context(&app, response, 400).await;
    }

    let database = app.database().await;
    assert_eq!(
        OidcAuthorizationTransactions::find()
            .count(&database)
            .await
            .expect("count transport-rejected snapshots"),
        0
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn exact_transport_limits_accept_valid_authorization_requests() {
    let app = spawn_preview("http://localhost:8000").await;
    let fixture = seed_authorization_client(&app, "https://client.example/callback", false).await;
    let path = authorize_path(&fixture);
    let client = no_redirect_client();

    let get_prefix = format!("{path}?");
    let get_query = pad_authorization_form_to_len(
        authorization_form(&fixture, "exact-target-state", "code", &[]),
        AUTHORIZATION_HTTP_LIMIT_BYTES - get_prefix.len(),
    );
    let get_target = format!("{get_prefix}{get_query}");
    assert_eq!(get_target.len(), AUTHORIZATION_HTTP_LIMIT_BYTES);
    let get_response = client
        .get(app.url(&get_target))
        .send()
        .await
        .expect("exact-limit GET authorization request");
    assert_success_snapshot(&app, &fixture, get_response, "exact-target-state", false).await;

    let post_body = pad_authorization_form_to_len(
        authorization_form(&fixture, "exact-body-state", "code", &[]),
        AUTHORIZATION_HTTP_LIMIT_BYTES,
    );
    assert_eq!(post_body.len(), AUTHORIZATION_HTTP_LIMIT_BYTES);
    let post_response = client
        .post(app.url(&path))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(post_body)
        .send()
        .await
        .expect("exact-limit POST authorization request");
    assert_success_snapshot(&app, &fixture, post_response, "exact-body-state", false).await;

    let database = app.database().await;
    assert_eq!(
        OidcAuthorizationTransactions::find()
            .count(&database)
            .await
            .expect("count exact-limit snapshots"),
        2
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn one_byte_over_transport_limits_reject_without_persistence() {
    let app = spawn_preview("http://localhost:8000").await;
    let fixture = seed_authorization_client(&app, "https://client.example/callback", false).await;
    let path = authorize_path(&fixture);
    let client = no_redirect_client();

    let get_prefix = format!("{path}?");
    let get_query = pad_authorization_form_to_len(
        authorization_form(&fixture, "over-target-state", "code", &[]),
        AUTHORIZATION_HTTP_LIMIT_BYTES + 1 - get_prefix.len(),
    );
    let get_target = format!("{get_prefix}{get_query}");
    assert_eq!(get_target.len(), AUTHORIZATION_HTTP_LIMIT_BYTES + 1);
    let get_response = client
        .get(app.url(&get_target))
        .send()
        .await
        .expect("one-byte-over GET authorization request");
    assert_rejected_without_success_context(&app, get_response, 414).await;

    let post_body = pad_authorization_form_to_len(
        authorization_form(&fixture, "over-body-state", "code", &[]),
        AUTHORIZATION_HTTP_LIMIT_BYTES + 1,
    );
    assert_eq!(post_body.len(), AUTHORIZATION_HTTP_LIMIT_BYTES + 1);
    let post_response = client
        .post(app.url(&path))
        .header("content-type", "application/x-www-form-urlencoded")
        .body(post_body)
        .send()
        .await
        .expect("one-byte-over POST authorization request");
    assert_rejected_without_success_context(&app, post_response, 413).await;

    let database = app.database().await;
    assert_eq!(
        OidcAuthorizationTransactions::find()
            .count(&database)
            .await
            .expect("count one-byte-over snapshots"),
        0
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn callback_trust_controls_local_and_redirected_protocol_errors() {
    let app = spawn_preview("http://localhost:8000").await;
    let fixture = seed_authorization_client(&app, "https://client.example/callback", false).await;
    let other = seed_authorization_client(&app, "https://other.example/callback", false).await;
    let client = no_redirect_client();

    let valid_form = authorization_form(&fixture, "tenant-boundary-state", "code", &[]);
    let invalid_tenant = client
        .get(app.url(&format!("/oidc/@@invalid@@/authorize?{valid_form}")))
        .send()
        .await
        .expect("invalid tenant request");
    assert!(invalid_tenant.headers().get("location").is_none());
    assert_rejected_without_success_context(&app, invalid_tenant, 400).await;

    for unavailable_sqid in [
        Sqid::from(Uuid::now_v7()).to_string(),
        Sqid::from(SYSTEM_TENANT_UUID).to_string(),
    ] {
        let unavailable_tenant = client
            .get(app.url(&format!("/oidc/{unavailable_sqid}/authorize?{valid_form}")))
            .send()
            .await
            .expect("unavailable tenant request");
        assert!(unavailable_tenant.headers().get("location").is_none());
        assert_rejected_without_success_context(&app, unavailable_tenant, 404).await;
    }

    let attacker_form = authorization_form(&fixture, "attacker-state", "code", &[])
        .replace(&fixture.client_id, "unknown-client")
        .replace(
            &form_urlencoded::byte_serialize(fixture.redirect_uri.as_bytes()).collect::<String>(),
            "https%3A%2F%2Fattacker.example%2Fcallback",
        );
    let unknown_client = client
        .get(app.url(&format!("{}?{attacker_form}", authorize_path(&fixture))))
        .send()
        .await
        .expect("unknown client request");
    assert!(unknown_client.headers().get("location").is_none());
    assert_rejected_without_success_context(&app, unknown_client, 400).await;

    let cross_tenant_form = authorization_form(&other, "cross-state", "code", &[]);
    let cross_tenant = client
        .get(app.url(&format!("{}?{cross_tenant_form}", authorize_path(&fixture))))
        .send()
        .await
        .expect("cross-tenant client request");
    assert!(cross_tenant.headers().get("location").is_none());
    assert_rejected_without_success_context(&app, cross_tenant, 400).await;

    let wrong_redirect_form = authorization_form(&fixture, "wrong-redirect", "code", &[])
        .replace("client.example%2Fcallback", "attacker.example%2Fcallback");
    let wrong_redirect = client
        .get(app.url(&format!(
            "{}?{wrong_redirect_form}",
            authorize_path(&fixture)
        )))
        .send()
        .await
        .expect("wrong redirect request");
    assert!(wrong_redirect.headers().get("location").is_none());
    assert_rejected_without_success_context(&app, wrong_redirect, 400).await;

    let state = "opaque state / + % value";
    let protocol_form = authorization_form(&fixture, state, "token", &[]);
    let protocol_error = client
        .get(app.url(&format!("{}?{protocol_form}", authorize_path(&fixture))))
        .send()
        .await
        .expect("trusted protocol error request");
    assert_eq!(protocol_error.status(), 303);
    assert_common_headers(&protocol_error);
    assert!(protocol_error.headers().get("set-cookie").is_none());
    let location = protocol_error
        .headers()
        .get("location")
        .expect("trusted callback error should redirect")
        .to_str()
        .expect("location should be ASCII");
    assert!(location.starts_with(&format!("{}?", fixture.redirect_uri)));
    let location = Url::parse(location).expect("protocol error Location should parse");
    let query: HashMap<_, _> = location.query_pairs().into_owned().collect();
    assert_eq!(
        query.get("error").map(String::as_str),
        Some("unsupported_response_type")
    );
    assert_eq!(query.get("state").map(String::as_str), Some(state));

    let request_object_form =
        authorization_form(&fixture, "request-state", "code", &[("request", "jwt")]);
    let request_object_error = client
        .get(app.url(&format!(
            "{}?{request_object_form}",
            authorize_path(&fixture)
        )))
        .send()
        .await
        .expect("unsupported request object");
    assert_eq!(request_object_error.status(), 303);
    let request_location = request_object_error
        .headers()
        .get("location")
        .expect("request object error Location")
        .to_str()
        .expect("ASCII Location");
    assert_eq!(
        Url::parse(request_location)
            .expect("request object Location")
            .query_pairs()
            .find(|(name, _)| name == "error")
            .map(|(_, value)| value.into_owned())
            .as_deref(),
        Some("request_not_supported")
    );

    let database = app.database().await;
    assert_eq!(
        OidcAuthorizationTransactions::find()
            .count(&database)
            .await
            .expect("count rejected snapshots"),
        0
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn deferred_commit_failure_returns_trusted_server_error_without_cookie_or_row() {
    let app = spawn_preview("http://localhost:8000").await;
    let fixture = seed_authorization_client(&app, "https://client.example/callback", false).await;
    let database = app.database().await;
    let schema = &app.schema_name;

    database
        .execute_unprepared(&format!(
            r#"
            CREATE FUNCTION "{schema}".test_reject_oidc_authorization_transaction_at_commit()
            RETURNS trigger
            LANGUAGE plpgsql
            AS $fixture$
            BEGIN
                RAISE EXCEPTION 'fixture deferred authorization transaction rejection';
            END;
            $fixture$
            "#
        ))
        .await
        .expect("create fixture-only deferred rejection function");
    database
        .execute_unprepared(&format!(
            r#"
            CREATE CONSTRAINT TRIGGER test_reject_oidc_authorization_transaction_at_commit
            AFTER INSERT ON "{schema}".oidc_authorization_transactions
            DEFERRABLE INITIALLY DEFERRED
            FOR EACH ROW
            EXECUTE FUNCTION "{schema}".test_reject_oidc_authorization_transaction_at_commit()
            "#
        ))
        .await
        .expect("create fixture-only deferred rejection trigger");

    let state = "commit failure / + % state";
    let response = no_redirect_client()
        .get(app.url(&format!(
            "{}?{}",
            authorize_path(&fixture),
            authorization_form(&fixture, state, "code", &[])
        )))
        .send()
        .await
        .expect("authorization request with deferred commit failure");
    assert_eq!(response.status(), 303);
    assert_common_headers(&response);
    assert!(response.headers().get("set-cookie").is_none());
    let location = response
        .headers()
        .get("location")
        .expect("trusted commit failure should redirect")
        .to_str()
        .expect("commit failure Location should be ASCII");
    assert_eq!(
        location,
        "https://client.example/callback?error=server_error&state=commit+failure+%2F+%2B+%25+state"
    );
    let query: HashMap<_, _> = Url::parse(location)
        .expect("commit failure Location should parse")
        .query_pairs()
        .into_owned()
        .collect();
    assert_eq!(query.get("error").map(String::as_str), Some("server_error"));
    assert_eq!(query.get("state").map(String::as_str), Some(state));
    assert_eq!(
        OidcAuthorizationTransactions::find()
            .count(&database)
            .await
            .expect("count snapshots after deferred commit rejection"),
        0
    );

    database
        .execute_unprepared(&format!(
            "DROP TRIGGER test_reject_oidc_authorization_transaction_at_commit ON \"{schema}\".oidc_authorization_transactions"
        ))
        .await
        .expect("drop fixture-only deferred rejection trigger");
    database
        .execute_unprepared(&format!(
            "DROP FUNCTION \"{schema}\".test_reject_oidc_authorization_transaction_at_commit()"
        ))
        .await
        .expect("drop fixture-only deferred rejection function");
}

// NOTE: AI-generated test
#[tokio::test]
async fn live_application_policy_overrides_a_stale_management_cache() {
    let app = spawn_preview("http://localhost:8000").await;
    let fixture = seed_authorization_client(&app, "http://localhost:3000/callback", true).await;

    let cached = app
        .state
        .applications
        .get_configuration(fixture.application_id)
        .await
        .expect("prime Application configuration cache");
    assert!(cached.oidc.allow_insecure_loopback_redirect_uris);

    let database = app.database().await;
    Applications::replace_configuration(
        fixture.application_id,
        ApplicationConfiguration::default(),
        &database,
    )
    .await
    .expect("disable loopback directly without invalidating cache");

    let response = no_redirect_client()
        .get(app.url(&format!(
            "{}?{}",
            authorize_path(&fixture),
            authorization_form(&fixture, "stale-cache-state", "code", &[])
        )))
        .send()
        .await
        .expect("authorization request under stale cache");
    assert!(response.headers().get("location").is_none());
    assert_rejected_without_success_context(&app, response, 400).await;
    assert_eq!(
        OidcAuthorizationTransactions::find()
            .count(&database)
            .await
            .expect("count stale-cache snapshots"),
        0
    );
}

// NOTE: AI-generated test
#[tokio::test]
async fn concurrent_duplicate_requests_create_distinct_rows_and_cookie_names() {
    let app = spawn_preview("http://localhost:8000").await;
    let fixture = seed_authorization_client(&app, "https://client.example/callback", false).await;
    let query = authorization_form(&fixture, "duplicate-state", "code", &[]);
    let url = app.url(&format!("{}?{query}", authorize_path(&fixture)));

    let (left, right) = tokio::join!(app.client.get(&url).send(), app.client.get(&url).send());
    let left = left.expect("first duplicate request");
    let right = right.expect("second duplicate request");
    assert_eq!(left.status(), 200);
    assert_eq!(right.status(), 200);
    let left_cookie_header = left
        .headers()
        .get("set-cookie")
        .expect("first binding cookie")
        .to_str()
        .expect("ASCII cookie");
    let right_cookie_header = right
        .headers()
        .get("set-cookie")
        .expect("second binding cookie")
        .to_str()
        .expect("ASCII cookie");
    assert!(!left_cookie_header.contains("; Secure"));
    assert!(!right_cookie_header.contains("; Secure"));
    let left_cookie = left_cookie_header
        .split('=')
        .next()
        .expect("cookie name")
        .to_owned();
    let right_cookie = right_cookie_header
        .split('=')
        .next()
        .expect("cookie name")
        .to_owned();
    assert_ne!(left_cookie, right_cookie);

    let database = app.database().await;
    assert_eq!(
        OidcAuthorizationTransactions::find()
            .count(&database)
            .await
            .expect("count duplicate snapshots"),
        2
    );
}

// NOTE: AI-generated test
#[test]
fn openapi_documents_both_authorization_entry_methods() {
    let specification = oceaniam::app::build_openapi_spec();
    let path = specification
        .paths
        .paths
        .get("/oidc/{tenant_sqid}/authorize")
        .expect("authorization preview path should be documented");
    assert!(path.get.is_some());
    assert!(path.post.is_some());
}
