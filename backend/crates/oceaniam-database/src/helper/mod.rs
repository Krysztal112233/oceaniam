#![doc = include_str!("./README.md")]

use crate::error::Error;
use oceaniam_vo::pagination::{PageInfo, PageParam, PagedResponse};
use sea_orm::{
    ConnectionTrait, EntityTrait, FromQueryResult, QuerySelect as _, Select, TransactionTrait,
};

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
