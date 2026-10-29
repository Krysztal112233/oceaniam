//! Durable PGMQ triggers for the worker consumer.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("SELECT pgmq.create('worker_jobs')")
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DO $$ BEGIN
                    IF EXISTS (SELECT 1 FROM pgmq.meta WHERE queue_name = 'worker_jobs') THEN
                        PERFORM pgmq.drop_queue('worker_jobs');
                    END IF;
                END $$",
            )
            .await?;
        Ok(())
    }
}
