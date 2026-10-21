// Rule 1 test exemption (`oceaniam-database/src/helper/README.md`): seeding and asserting
// state with SeaORM builders is the legitimate job of tests.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

mod support;

#[path = "./api/mod.rs"]
mod api;
