pub mod client;
pub mod error;
pub mod paths;

mod administrators;
mod app_users;
mod applications;
mod auth;
mod keys;
mod secrets;
mod statistics;
mod tenants;

pub use client::OceanIamClient;

pub use error::Error;

// Re-export response types
pub use oceaniam_vo::pagination::{PageInfo, PageParam, PagedResponse};
pub use oceaniam_vo::response::{Empty, ErrorResponse};

// Re-export VO types
pub use oceaniam_vo::administrators::*;
pub use oceaniam_vo::applications::*;
pub use oceaniam_vo::auth::*;
pub use oceaniam_vo::statistics::*;
pub use oceaniam_vo::tenants::*;
