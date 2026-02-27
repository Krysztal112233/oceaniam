use sea_orm::sea_query::extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(KeyAlgType::KeyAlg)
                    .values([KeyAlgType::Rs256])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(KeyStatusEnum::KeyStatus)
                    .values([
                        KeyStatusEnum::Active,
                        KeyStatusEnum::Pending,
                        KeyStatusEnum::Retired,
                        KeyStatusEnum::Revoked,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(KeyBoxes::Table)
                    .col(pk_uuid(KeyBoxes::ID))
                    .col(uuid_uniq(KeyBoxes::Kid))
                    .col(enumeration(
                        KeyBoxes::KeyAlg,
                        KeyAlgType::KeyAlg,
                        [KeyAlgType::Rs256],
                    ))
                    .col(enumeration(
                        KeyBoxes::Status,
                        KeyStatusEnum::KeyStatus,
                        [
                            KeyStatusEnum::Active,
                            KeyStatusEnum::Pending,
                            KeyStatusEnum::Retired,
                            KeyStatusEnum::Revoked,
                        ],
                    ))
                    .col(timestamp_with_time_zone(KeyBoxes::CreatedAt))
                    .col(timestamp_with_time_zone_null(KeyBoxes::ActivatedAt))
                    .col(timestamp_with_time_zone_null(KeyBoxes::RetiredAt))
                    .col(timestamp_with_time_zone_null(KeyBoxes::RevokedAt))
                    .col(timestamp_with_time_zone_null(KeyBoxes::ExpiresAt))
                    .col(json_binary(KeyBoxes::Secret))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(KeyBoxes::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(KeyStatusEnum::KeyStatus).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(KeyAlgType::KeyAlg).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum KeyStatusEnum {
    KeyStatus,

    Active,
    Pending,
    Retired,
    Revoked,
}

#[derive(DeriveIden)]
enum KeyAlgType {
    KeyAlg,

    Rs256,
}

#[derive(DeriveIden)]
enum KeyBoxes {
    Table,

    ID,
    KeyAlg,
    Kid,
    Status,
    CreatedAt,
    ActivatedAt,
    RetiredAt,
    RevokedAt,
    ExpiresAt,
    Secret,
}
