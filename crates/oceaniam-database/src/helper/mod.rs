use sea_orm::{ConnectionTrait, TransactionTrait};

pub mod applications;
pub mod credentials;
pub mod subjects;
pub mod users;

pub trait SafeTransactionConnectionTrait: TransactionTrait + ConnectionTrait + Sync + Send {}

impl<T> SafeTransactionConnectionTrait for T where
    T: TransactionTrait + ConnectionTrait + Send + Sync
{
}
