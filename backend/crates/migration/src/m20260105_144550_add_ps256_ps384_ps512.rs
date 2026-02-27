use sea_orm::sea_query::extension::postgres::Type;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for alg in [KeyAlgType::PS256, KeyAlgType::PS384, KeyAlgType::PS512] {
            manager
                .alter_type(
                    Type::alter()
                        .name(KeyAlgType::KeyAlg)
                        .add_value(alg)
                        .if_not_exists(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(DeriveIden)]
enum KeyAlgType {
    KeyAlg,

    PS256,
    PS384,
    PS512,
}
