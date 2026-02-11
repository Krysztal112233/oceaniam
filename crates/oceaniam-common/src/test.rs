use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseTransaction, DbErr, TransactionTrait};

use crate::error::Error;

#[derive(Debug)]
pub struct TestDatabaseTransaction(DatabaseTransaction);

impl std::ops::Deref for TestDatabaseTransaction {
    type Target = DatabaseTransaction;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TestDatabaseTransaction {
    /// Creates a new TestDatabaseTransaction by connecting to the test database and starting a new
    /// transaction.
    ///
    /// # Note
    ///
    /// This function will run [Migrator::up] automatically.
    ///
    /// # Returns
    /// * `Ok(TestDatabaseTransaction)` on successful connection and transaction start
    /// * `Err(Error)` if connection or transaction fails
    pub async fn with_opt<C>(opt: C) -> Result<Self, Error>
    where
        C: Into<ConnectOptions>,
    {
        let db = Database::connect(opt).await?;
        let txn = db.begin().await?;

        {
            let db = txn.begin().await?;
            Migrator::up(&db, None).await?;
        }

        Ok(Self(txn))
    }

    /// Executes an async closure to preset database content before testing.
    ///
    /// This method accepts an async closure that receives a reference to the internal
    /// `DatabaseTransaction`, allowing you to insert, update, or delete data as needed
    /// for your test setup.
    ///
    /// # Type Parameters
    /// * `F` - The async closure type
    /// * `Fut` - The future type returned by the closure
    ///
    /// # Parameters
    /// * `f` - An async closure that takes `&DatabaseTransaction` and returns `Result<(), Error>`
    ///
    /// # Returns
    /// * `Ok(())` if the preset operation succeeds
    /// * `Err` if the preset operation fails
    ///
    pub async fn preset<F, Fut>(&self, f: F) -> Result<(), Error>
    where
        F: FnOnce(&DatabaseTransaction) -> Fut,
        Fut: Future<Output = Result<(), Error>>,
    {
        f(&self.0).await
    }

    pub async fn start(&self) -> Result<TestDatabaseTransaction, DbErr> {
        Ok(Self(self.0.begin().await?))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_database_initialize() -> Result<(), Error> {
        let _ = TestDatabaseTransaction::with_opt(
            "postdresql://postgres:postgres@localhost:5432/postgres",
        )
        .await?;

        Ok(())
    }
}
