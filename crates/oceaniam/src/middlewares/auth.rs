use axum::{
    extract::FromRequestParts,
    http::{StatusCode, header, request::Parts},
};
use jsonwebtoken::{DecodingKey, Header, TokenData, Validation, decode, decode_header};
use oceaniam_common::{error::Error, jwks::JwkSet};
use serde::{Serialize, de::DeserializeOwned};
use uuid::Uuid;

use crate::{keybox::ApplicationKeyBoxManager, state::AppState};

pub struct RequireAuth {
    token: String,
    header: Header,
}

impl RequireAuth {
    pub async fn validate<S>(
        self,
        application_id: Uuid,

        keybox: &mut ApplicationKeyBoxManager,
        validation: &Validation,
    ) -> Result<TokenData<S>, Error>
    where
        S: Serialize + DeserializeOwned,
    {
        let Some(key) = keybox.get_keybox(application_id).await else {
            return Err(Error::with_code(
                StatusCode::BAD_REQUEST,
                format!("cannot find keybox of `{application_id}`"),
            ));
        };

        let key = {
            // NOTE: We have already proved that `kid` must exist.
            let kid = &self.header.kid.unwrap();
            let tmp = jsonwebtoken::jwk::JwkSet::from(JwkSet::from(key));
            let Some(jwk) = tmp.find(kid) else {
                return Err(Error::with_code(
                    StatusCode::BAD_REQUEST,
                    format!("cannot find jwk for kid `{kid}`."),
                ));
            };

            DecodingKey::from_jwk(jwk)?
        };

        Ok(decode(self.token, &key, validation)?)
    }
}

impl FromRequestParts<AppState> for RequireAuth {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        AppState { .. }: &AppState,
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

        Ok(RequireAuth { token, header })
    }
}
