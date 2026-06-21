use axum_extra::extract::cookie::{Cookie, SameSite};
use time::Duration;

/// Build a signed auth cookie with security attributes.
pub fn build_auth_cookie(jwt: &str, secure: bool) -> Cookie<'static> {
    Cookie::build(("auth_token", jwt.to_owned()))
        .secure(secure)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(Duration::seconds(24 * 5 * 3600))
        .build()
}

/// Build a cookie that clears the auth token on the client.
pub fn clear_auth_cookie(secure: bool) -> Cookie<'static> {
    Cookie::build(("auth_token", ""))
        .secure(secure)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(Duration::ZERO)
        .build()
}
