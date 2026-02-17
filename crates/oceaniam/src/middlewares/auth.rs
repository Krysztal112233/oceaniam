use axum::{
    extract::FromRequestParts,
    http::{StatusCode, header, request::Parts},
};
use jsonwebtoken::{DecodingKey, Header, TokenData, Validation, decode, decode_header};
use oceaniam_common::{error::Error, jwks::JwkSet};
use serde::{Serialize, de::DeserializeOwned};

use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct RequireAuth<S> {
    pub token: TokenData<S>,
    pub header: Header,
}

pub async fn validate<S>(
    header: &Header,
    jwks: JwkSet,
    token: String,
    validation: &Validation,
) -> Result<TokenData<S>, Error>
where
    S: Serialize + DeserializeOwned,
{
    let key = {
        // NOTE: We have already proved that `kid` must exist.
        let kid = header.kid.as_ref().unwrap();
        let tmp = jsonwebtoken::jwk::JwkSet::from(jwks);
        let Some(jwk) = tmp.find(kid) else {
            return Err(Error::with_code(
                StatusCode::BAD_REQUEST,
                format!("cannot find jwk for kid `{kid}`."),
            ));
        };

        DecodingKey::from_jwk(jwk)?
    };

    Ok(decode(token, &key, validation)?)
}

impl<S> FromRequestParts<AppState> for RequireAuth<S>
where
    S: Serialize + DeserializeOwned,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        AppState {
            system_jwks,
            jwt_validation,
            ..
        }: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok());

        let Some(token) = auth_header else {
            return Err(StatusCode::NON_AUTHORITATIVE_INFORMATION);
        };

        if !token.starts_with("Bearer ") {
            return Err(StatusCode::BAD_REQUEST);
        }

        let token = token.trim().replace("Bearer ", "");

        let Ok(header) = decode_header(&token) else {
            return Err(StatusCode::BAD_REQUEST);
        };

        // NOTE: By design, token's header must have `kid` field.
        if header.kid.is_none() {
            return Err(StatusCode::BAD_REQUEST);
        }

        let Ok(token) = validate(&header, system_jwks.jwks(), token, jwt_validation).await else {
            return Err(StatusCode::BAD_REQUEST);
        };

        Ok(Self { token, header })
    }
}
