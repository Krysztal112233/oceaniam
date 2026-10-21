// Rule 1 test exemption (`oceaniam-database/src/helper/README.md`): seeding and asserting
// state with SeaORM builders is the legitimate job of tests.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

mod support;

#[path = "./database/key_box.rs"]
mod key_box;

#[path = "./database/audit.rs"]
mod audit;

#[path = "./database/oidc_authorization_codes.rs"]
mod oidc_authorization_codes;

#[path = "./database/oidc_authorization_transactions.rs"]
mod oidc_authorization_transactions;

#[path = "./database/oidc_clients.rs"]
mod oidc_clients;

#[path = "./database/hardening.rs"]
mod hardening;

#[path = "./database/time_authority.rs"]
mod time_authority;
