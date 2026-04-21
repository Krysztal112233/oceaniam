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
                    .as_enum(ChallengeFactorTypeEnum::ChallengeFactorType)
                    .values([ChallengeFactorTypeEnum::Totp])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(ChallengePurposeTypeEnum::ChallengePurposeType)
                    .values([ChallengePurposeTypeEnum::Signin])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(ChallengeStatusTypeEnum::ChallengeStatusType)
                    .values([
                        ChallengeStatusTypeEnum::Pending,
                        ChallengeStatusTypeEnum::Consumed,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Challenges::Table)
                    .if_not_exists()
                    .col(pk_uuid(Challenges::Id))
                    .col(uuid(Challenges::ApplicationId).not_null())
                    .col(uuid(Challenges::SubjectId).not_null())
                    .col(string(Challenges::Token).not_null())
                    .col(enumeration(
                        Challenges::FactorType,
                        ChallengeFactorTypeEnum::ChallengeFactorType,
                        [ChallengeFactorTypeEnum::Totp],
                    ))
                    .col(enumeration(
                        Challenges::Purpose,
                        ChallengePurposeTypeEnum::ChallengePurposeType,
                        [ChallengePurposeTypeEnum::Signin],
                    ))
                    .col(enumeration(
                        Challenges::Status,
                        ChallengeStatusTypeEnum::ChallengeStatusType,
                        [
                            ChallengeStatusTypeEnum::Pending,
                            ChallengeStatusTypeEnum::Consumed,
                        ],
                    ))
                    .col(integer(Challenges::AttemptCount).not_null().default(0))
                    .col(integer(Challenges::MaxAttempts).not_null().default(5))
                    .col(timestamp_with_time_zone(Challenges::ExpiresAt).not_null())
                    .col(timestamp_with_time_zone_null(Challenges::ConsumedAt))
                    .col(
                        timestamp_with_time_zone(Challenges::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Challenges::Table).to_owned())
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(ChallengeStatusTypeEnum::ChallengeStatusType)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(ChallengePurposeTypeEnum::ChallengePurposeType)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_type(
                Type::drop()
                    .name(ChallengeFactorTypeEnum::ChallengeFactorType)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Challenges {
    Table,

    Id,

    ApplicationId,
    SubjectId,

    Token,
    FactorType,
    Purpose,
    Status,
    AttemptCount,
    MaxAttempts,
    ExpiresAt,
    ConsumedAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum ChallengeFactorTypeEnum {
    ChallengeFactorType,

    Totp,
}

#[derive(DeriveIden)]
enum ChallengePurposeTypeEnum {
    ChallengePurposeType,

    Signin,
}

#[derive(DeriveIden)]
enum ChallengeStatusTypeEnum {
    ChallengeStatusType,

    Pending,
    Consumed,
}
