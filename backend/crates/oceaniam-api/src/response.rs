use axum::{
    Json,
    http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{InvalidHeaderValue, SET_COOKIE},
    },
    response::IntoResponse,
};
use axum_extra::extract::cookie::Cookie;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use oceaniam_common::error::Error;

#[derive(Debug, Deserialize, Serialize, Default, ToSchema)]
pub struct Empty {}

#[derive(Debug, Deserialize, Serialize, Default, ToSchema)]
#[serde(transparent)]
pub struct StatusCodeOnlyResponse(ApiResponse<Empty>);

#[derive(Debug, Deserialize, Serialize, Default, ToSchema, TS)]
pub struct ErrorResponse {
    msg: String,
}

impl ErrorResponse {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { msg: msg.into() }
    }
}

#[derive(Debug, Deserialize, Serialize, Default, ToSchema)]
pub struct ApiResponse<T> {
    #[serde(flatten)]
    payload: Option<T>,
}

pub type RestResult<T> = ::std::result::Result<ApiResponse<T>, Error>;
pub type WithHeaderRestResult<T> = ::std::result::Result<ApiResponseWithHeader<T>, Error>;

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn new(payload: T) -> Self {
        Self {
            payload: Some(payload),
        }
    }
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Debug, Deserialize, Serialize, Default, ToSchema)]
pub struct ApiResponseWithHeader<T> {
    #[serde(flatten)]
    payload: Option<T>,

    #[serde(skip)]
    headers: HeaderMap,
}

impl<T> ApiResponseWithHeader<T>
where
    T: Serialize,
{
    pub fn new(payload: T) -> Self {
        Self {
            payload: Some(payload),
            headers: HeaderMap::new(),
        }
    }

    pub fn empty() -> Self {
        Self {
            payload: None,
            headers: HeaderMap::new(),
        }
    }

    pub fn with_header(payload: T, headers: HeaderMap) -> Self {
        Self {
            payload: Some(payload),
            headers,
        }
    }

    pub fn set_cookie(&mut self, cookie: Cookie<'static>) -> Result<(), Error> {
        let header_value: HeaderValue = cookie
            .encoded()
            .to_string()
            .parse::<HeaderValue>()
            .map_err(|e| Error::with_code(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        self.headers.append(SET_COOKIE, header_value);
        Ok(())
    }

    pub fn with_cookie(mut self, cookie: Cookie<'static>) -> Result<Self, Error> {
        self.set_cookie(cookie)?;
        Ok(self)
    }

    pub fn set_cookie_str(&mut self, cookie: impl AsRef<str>) -> Result<(), InvalidHeaderValue> {
        let header_value: HeaderValue = cookie.as_ref().parse()?;
        self.headers.append(SET_COOKIE, header_value);
        Ok(())
    }

    pub fn with_cookie_str(mut self, cookie: impl AsRef<str>) -> Result<Self, InvalidHeaderValue> {
        self.set_cookie_str(cookie)?;
        Ok(self)
    }
}

impl<T> IntoResponse for ApiResponseWithHeader<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        let Self { payload, headers } = self;

        let mut response = (
            StatusCode::OK,
            Json(Self {
                payload,
                headers: HeaderMap::new(),
            }),
        )
            .into_response();

        response.headers_mut().extend(headers);
        response
    }
}

#[cfg(test)]
mod tests {
    use super::{ApiResponseWithHeader, Empty};

    use axum::http::header::SET_COOKIE;
    use axum::response::IntoResponse;
    use axum_extra::extract::cookie::Cookie;

    // NOTE: AI-generated test
    #[test]
    fn with_cookie_str_appends_set_cookie_header() {
        let response = ApiResponseWithHeader::new(Empty::default())
            .with_cookie_str("a=b; Path=/")
            .unwrap()
            .into_response();

        assert_eq!(response.headers().get_all(SET_COOKIE).iter().count(), 1);
    }

    // NOTE: AI-generated test
    #[test]
    fn with_cookie_appends_multiple_set_cookie_headers() {
        let response = ApiResponseWithHeader::new(Empty::default())
            .with_cookie(Cookie::new("a", "1"))
            .unwrap()
            .with_cookie(Cookie::new("b", "2"))
            .unwrap()
            .into_response();

        assert_eq!(response.headers().get_all(SET_COOKIE).iter().count(), 2);
    }

    // NOTE: AI-generated test
    #[test]
    fn invalid_cookie_str_returns_err() {
        let result = ApiResponseWithHeader::new(Empty::default()).with_cookie_str("a=b\r\nbad: 1");
        assert!(result.is_err());
    }
}
