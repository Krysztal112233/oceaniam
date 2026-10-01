#![doc = include_str!("./README.md")]

use crate::error::Error;
use chrono::{DateTime, FixedOffset};
use oceaniam_vo::pagination::{PageInfo, PageParam, PagedResponse};
use sea_orm::{
    ConnectionTrait, DatabaseBackend, EntityTrait, FromQueryResult, QuerySelect as _, Select,
    Statement, TransactionTrait,
};

pub mod administrator_tenants;
pub mod administrators;
pub mod application_roles;
pub mod applications;
pub mod applications_secrets;
pub mod audit_summary_by_application;
pub mod audits;
pub mod challenges;
pub mod credentials;
pub mod key_boxes;
pub mod macros;
pub mod oidc_authorization_entry;
pub mod oidc_authorization_transactions;
pub mod oidc_clients;
pub mod pgmq;
pub mod revoked_jwts;
pub mod role_permissions;
pub mod statistics;
pub mod subject_roles;
pub mod subjects;
pub mod tenants;
pub mod trend;
pub mod users;

pub trait SafeTransactionConnectionTrait: TransactionTrait + ConnectionTrait + Sync + Send {}

impl<T> SafeTransactionConnectionTrait for T where
    T: TransactionTrait + ConnectionTrait + Send + Sync
{
}

/// Samples the database wall clock (`clock_timestamp()`), the single time authority for any
/// timestamp persisted to the database or evaluated by a database predicate (Rule 3).
///
/// Unlike `CURRENT_TIMESTAMP`, `clock_timestamp()` is not pinned to transaction start, so the
/// sample reflects the moment the statement executes.
pub async fn db_clock_now(
    database: &impl SafeTransactionConnectionTrait,
) -> Result<DateTime<FixedOffset>, Error> {
    database
        .query_one_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT clock_timestamp() AS now".to_owned(),
        ))
        .await?
        .ok_or(Error::from(sea_orm::DbErr::RecordNotFound(
            "clock_timestamp".to_owned(),
        )))?
        .try_get("", "now")
        .map_err(Into::into)
}

/// Collapses the rows of a primary-key-filtered `exec_with_returning` write into exactly one
/// row, mapping an empty result to the caller's not-found error.
///
/// A write filtered by primary key can return at most one row; the `debug_assert!` keeps that
/// invariant loud in debug builds instead of silently discarding rows. Callers pass their
/// entity-specific not-found constructor so each site keeps its established error shape.
pub fn exactly_one_returned<M>(
    mut rows: Vec<M>,
    not_found: impl FnOnce() -> Error,
) -> Result<M, Error> {
    debug_assert!(
        rows.len() <= 1,
        "a primary-key-filtered write returned multiple rows"
    );
    rows.pop().ok_or_else(not_found)
}

pub trait PagedSelect {
    fn paged(self, paged: PageParam) -> Self;
}

impl<T> PagedSelect for Select<T>
where
    T: EntityTrait,
{
    fn paged(self, page: PageParam) -> Self {
        self.offset(page.as_offset()).limit(page.as_limit())
    }
}

#[async_trait::async_trait]
pub trait PagedExecutor<'db, C: ConnectionTrait, S: sea_orm::SelectorTrait> {
    async fn fetch_paged(
        self,
        page: PageParam,
    ) -> Result<PagedResponse<<S as sea_orm::SelectorTrait>::Item>, Error>;
}

#[async_trait::async_trait]
impl<'db, C, S> PagedExecutor<'db, C, S> for sea_orm::Paginator<'db, C, S>
where
    C: ConnectionTrait + Sync + Send,
    S: sea_orm::SelectorTrait + Send + Sync,
    <S as sea_orm::SelectorTrait>::Item: FromQueryResult + Send + Sync,
{
    #[tracing::instrument(
        level = "info",
        name = "db.pagination.fetch_paged",
        skip_all,
        fields(otel.kind = "internal")
    )]
    async fn fetch_paged(
        self,
        page: PageParam,
    ) -> Result<PagedResponse<<S as sea_orm::SelectorTrait>::Item>, Error> {
        let page_num = page.page.saturating_sub(1);
        let items = self.fetch_page(page_num).await?;
        let total = self.num_items().await? as usize;
        let has_next = (page.as_offset() + items.len() as u64) < total as u64;

        Ok(PagedResponse {
            items,
            page_info: PageInfo { has_next, total },
        })
    }
}
