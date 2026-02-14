use oceaniam_common::PageParam;
use sea_orm::{ConnectionTrait, EntityTrait, QuerySelect as _, Select, TransactionTrait};

pub mod applications;
pub mod credentials;
pub mod key_boxes;
pub mod subjects;
pub mod tenants;
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
