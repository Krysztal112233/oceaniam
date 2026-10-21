// Rule 1 enforcement (`oceaniam-database/src/helper/README.md`): business src/ code must not
// use SeaORM query builders directly; the disallowed lists live in this crate's clippy.toml.
#![deny(clippy::disallowed_methods, clippy::disallowed_types)]

pub mod app;
pub(crate) mod conversion;
pub mod endpoints;
pub mod error;
#[allow(unused)]
pub(crate) mod middlewares;
pub(crate) mod setup;
pub mod state;
pub(crate) mod util;
