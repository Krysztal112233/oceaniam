//! Independent RP interoperability against the real, per-instance issuer and advertised URLs.
//! Only browser-form interaction uses the provider fixtures; discovery, PKCE, token exchange,
//! ID-token verification and UserInfo use the unmodified `openidconnect` client.

use std::time::Duration;

use oceaniam_database::{helper::users::UserHelper, model::prelude::Users};
use openidconnect::{
    AccessTokenHash, AuthType, AuthorizationCode, ClaimsVerificationError, ClientId, CsrfToken,
    HttpRequest, HttpResponse, IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge,
    RedirectUrl, TokenResponse,
    core::{
        CoreAuthenticationFlow, CoreClient, CoreIdTokenVerifier, CoreJwsSigningAlgorithm,
        CoreProviderMetadata, CoreUserInfoClaims,
    },
};
use reqwest::{Client, redirect::Policy};
use url::Url;

use crate::support::{
    oidc_drive::{
        USER_PASSWORD, assert_challenge_form, assert_code_redirect, challenge_form,
        create_mfa_user, create_user, drive_entry_url, login_form, post_challenge, post_login,
        seed_login_fixture,
    },
    spawn_app_with_isolated_schema_configured,
};

/// Transport only: oauth2 5's built-in reqwest adapter targets 0.12, while this workspace uses
/// 0.13. Pass request/response bytes and headers through unchanged, without endpoint rewriting.
async fn oidc_http(client: &Client, request: HttpRequest) -> Result<HttpResponse, reqwest::Error> {
    let response = client.execute(request.try_into()?).await?;
    let mut builder = openidconnect::http::Response::builder().status(response.status());
    *builder.headers_mut().expect("response headers") = response.headers().clone();
    Ok(builder
        .body(response.bytes().await?.to_vec())
        .expect("valid HTTP response"))
}

// NOTE: AI-generated test
#[tokio::test]
async fn independent_client_verifies_password_and_totp_code_flows() {
    let app = spawn_app_with_isolated_schema_configured(|config| {
        config.oidc.authorization_entry_preview_enabled = true;
        config.public_base_url = Url::parse(&format!("http://{}", config.addr))
            .expect("bound test origin")
            .try_into()
            .expect("loopback origin supports the preview");
    })
    .await;
    let fixture = seed_login_fixture(&app).await;
    let issuer = IssuerUrl::new(app.url(&format!("/oidc/{}", fixture.tenant_sqid)))
        .expect("canonical issuer");
    let http = Client::builder()
        .redirect(Policy::none())
        .timeout(Duration::from_secs(10))
        .no_proxy()
        .build()
        .expect("independent client transport");
    let transport = |request| oidc_http(&http, request);
    let metadata = CoreProviderMetadata::discover_async(issuer.clone(), &transport)
        .await
        .expect("independent discovery and advertised JWKS retrieval");
    assert_eq!(metadata.issuer(), &issuer);
    assert!(!metadata.jwks().keys().is_empty());
    assert_eq!(
        metadata.id_token_signing_alg_values_supported(),
        &[CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256]
    );
    let wrong_audience = CoreIdTokenVerifier::new_public_client(
        ClientId::new("unregistered-client".to_owned()),
        issuer.clone(),
        metadata.jwks().clone(),
    );
    let client = CoreClient::from_provider_metadata(
        metadata,
        ClientId::new(fixture.client_id.clone()),
        None,
    )
    .set_auth_type(AuthType::RequestBody)
    .set_redirect_uri(RedirectUrl::new(fixture.redirect_uri.clone()).expect("registered callback"));

    let password_email = "interop-password@example.com";
    let password_user = create_user(&app, &fixture, password_email, USER_PASSWORD).await;
    let mfa_user = create_mfa_user(&app, &fixture, "interop-totp@example.com").await;
    let database = app.database().await;
    for (user_id, email, mfa) in [
        (password_user, password_email, None),
        (mfa_user.user_id, mfa_user.email.as_str(), Some(&mfa_user)),
    ] {
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let (authorization_url, state, nonce) = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .set_pkce_challenge(challenge)
            .url();
        let entry = drive_entry_url(&app, &fixture, authorization_url).await;
        let mut response = post_login(
            &app,
            &fixture,
            &entry,
            login_form(email, USER_PASSWORD, &entry.csrf),
        )
        .await;
        if let Some(mfa) = mfa {
            let page = assert_challenge_form(response, &fixture, &entry, state.secret()).await;
            response = post_challenge(
                &app,
                &fixture,
                &entry,
                challenge_form(&mfa.current_code(), &page.csrf),
            )
            .await;
        }
        // Validates the registered callback, state round-trip, and authorization response issuer.
        // The external callback is captured, not followed, just as a relying party would do.
        let redirect = assert_code_redirect(response, &fixture, &entry, state.secret()).await;
        assert_eq!(redirect.iss, issuer.as_str());
        let tokens = client
            .exchange_code(AuthorizationCode::new(redirect.code))
            .expect("discovered token endpoint")
            .set_pkce_verifier(verifier)
            .request_async(&transport)
            .await
            .unwrap_or_else(|_| panic!("independent public-client PKCE exchange failed"));
        assert!(tokens.refresh_token().is_none());
        let id_token = tokens
            .id_token()
            .expect("OIDC token response includes ID token");
        let id_verifier = client.id_token_verifier();
        let claims = id_token
            .claims(&id_verifier, &nonce)
            .expect("signature, advertised algorithm, issuer, audience, expiry and nonce verify");
        assert_eq!(
            id_token.signing_alg().expect("signed ID token"),
            &CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256
        );
        let user = Users::get_user_of_application(fixture.application_id, user_id, &database)
            .await
            .expect("fixture user");
        assert_eq!(claims.subject().as_str(), user.oidc_sub.to_string());
        assert!(claims.auth_time().is_some());
        if let Some(expected_hash) = claims.access_token_hash() {
            let actual_hash = AccessTokenHash::from_token(
                tokens.access_token(),
                id_token.signing_alg().expect("signing algorithm"),
                id_token
                    .signing_key(&id_verifier)
                    .expect("discovered signing key"),
            )
            .expect("access token hash");
            assert_eq!(&actual_hash, expected_hash);
        }

        // Sanity-check fail-closed client validation using this genuinely issued token.
        assert!(matches!(
            id_token.claims(&id_verifier, &Nonce::new("wrong-nonce".to_owned())),
            Err(ClaimsVerificationError::InvalidNonce(_))
        ));
        assert!(matches!(
            id_token.claims(&wrong_audience, &nonce),
            Err(ClaimsVerificationError::InvalidAudience(_))
        ));
        let expired_at = claims.expiration();
        let expired_verifier = id_verifier.clone().set_time_fn(move || expired_at);
        assert!(matches!(
            id_token.claims(&expired_verifier, &nonce),
            Err(ClaimsVerificationError::Expired(_))
        ));

        let userinfo: CoreUserInfoClaims = client
            .user_info(
                tokens.access_token().clone(),
                Some(claims.subject().clone()),
            )
            .expect("discovered UserInfo endpoint")
            .request_async(&transport)
            .await
            .unwrap_or_else(|_| {
                panic!("independent UserInfo request or subject verification failed")
            });
        assert_eq!(
            serde_json::to_value(userinfo).expect("UserInfo claims"),
            serde_json::json!({"sub": user.oidc_sub.to_string()})
        );
    }
}
