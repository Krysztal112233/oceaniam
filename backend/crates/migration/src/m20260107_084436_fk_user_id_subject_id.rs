use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const FK_USER_ID_SUBJECT_ID: &str = "fk_user_id_subject_id";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_USER_ID_SUBJECT_ID)
                    .from(Users::Table, Users::ID)
                    .to(Subjects::Table, Subjects::ID)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(Users::Table)
                    .name(FK_USER_ID_SUBJECT_ID)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Subjects {
    Table,

    ID,
}

#[derive(DeriveIden)]
enum Users {
    Table,

    ID,
}
