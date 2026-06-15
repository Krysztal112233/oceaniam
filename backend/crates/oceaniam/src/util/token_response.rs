use oceaniam_api::ApiResponse;
use oceaniam_vo::auth::{SigninResponseOrChallenge, SignupResponse};

use crate::{
    error::Error, middlewares::auth::TokenDispatchMethodGuard, util::cookie::build_auth_cookie,
};

/// Builds a signin response that returns the JWT in JSON, an auth cookie, or both.
pub fn dispatch_signin_response(
    jwt: String,
    token_mtd: &TokenDispatchMethodGuard,
    secure_cookie: bool,
) -> Result<ApiResponse<SigninResponseOrChallenge>, Error> {
    let cookie = build_auth_cookie(&jwt, secure_cookie);
    let resp = ApiResponse::new(SigninResponseOrChallenge::Signup(SignupResponse { jwt }));

    Ok(match token_mtd {
        TokenDispatchMethodGuard::Cookie => ApiResponse::empty().with_cookie(cookie)?,
        TokenDispatchMethodGuard::Json => resp,
        TokenDispatchMethodGuard::Both => resp.with_cookie(cookie)?,
    })
}
