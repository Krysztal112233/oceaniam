use sea_orm::sea_query::extension::postgres::Type;
use sea_orm_migration::{
    prelude::{extension::postgres::TypeCreateStatement, *},
    schema::*,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

const FK_CREDENTIAL_SUBJECT: &str = "fk_credential_id_subject_ref_id";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .col(pk_uuid(Users::ID))
                    .col(string(Users::Name))
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(SubjectType::SubjectTypeEnum)
                    .values([SubjectType::User])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Subjects::Table)
                    .col(pk_uuid(Subjects::RefID))
                    .col(enumeration(
                        Subjects::Type,
                        SubjectType::SubjectTypeEnum,
                        [SubjectType::User],
                    ))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Credentials::Table)
                    .col(pk_uuid(Credentials::SubjectID))
                    .col(json_binary(Credentials::Value))
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .from(Credentials::Table, Credentials::SubjectID)
                    .to(Subjects::Table, Subjects::RefID)
                    .name(FK_CREDENTIAL_SUBJECT)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(Credentials::Table)
                    .name(FK_CREDENTIAL_SUBJECT)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Subjects::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Credentials::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(SubjectType::SubjectTypeEnum).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,

    ID,
    Name,
}

#[derive(DeriveIden)]
enum SubjectType {
    SubjectTypeEnum,

    User,
}

#[derive(DeriveIden)]
enum Subjects {
    Table,

    RefID,
    Type,
}

#[derive(DeriveIden)]
enum Credentials {
    Table,

    SubjectID,
    Value,
}
