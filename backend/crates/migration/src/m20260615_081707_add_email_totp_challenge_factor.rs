use sea_orm::sea_query::extension::postgres::Type;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_type(
                Type::alter()
                    .name(ChallengeFactorTypeEnum::ChallengeFactorType)
                    .add_value(ChallengeFactorTypeEnum::EmailTotp)
                    .if_not_exists(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(DeriveIden)]
enum ChallengeFactorTypeEnum {
    ChallengeFactorType,

    EmailTotp,
}
