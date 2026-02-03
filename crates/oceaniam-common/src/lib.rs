use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::error::Error;

pub mod config;
pub mod error;
pub mod jwt;

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

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PageInfo {
    pub has_next: bool,
    pub total: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PagedResponse<T> {
    pub items: Vec<T>,
    pub page_info: PageInfo,
}

impl<T> PagedResponse<T> {
    pub fn with_entire<I>(data: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let items = data.into_iter().collect::<Vec<_>>();

        let page_info = PageInfo {
            has_next: false,
            total: items.len(),
        };

        Self { items, page_info }
    }
}
