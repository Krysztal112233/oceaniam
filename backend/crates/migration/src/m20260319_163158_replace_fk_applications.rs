use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const FK_USER_APPLICATION: &str = "fk_user_id_application_id";
const FK_SUBJECT_APPLICATION: &str = "fk_subjects_id_application_id";
const FK_KEY_BOXES_APPLICATION: &str = "fk_key_boxes_id_application_id";
const FK_APPLICATION_SECRET_APPLICATION: &str = "fk_application_secret_application_id";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(Users::Table)
                    .name(FK_USER_APPLICATION)
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_USER_APPLICATION)
                    .from(Users::Table, Users::ApplicationID)
                    .to(Applications::Table, Applications::ID)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(Subjects::Table)
                    .name(FK_SUBJECT_APPLICATION)
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_SUBJECT_APPLICATION)
                    .from(Subjects::Table, Subjects::ApplicationID)
                    .to(Applications::Table, Applications::ID)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(KeyBoxes::Table)
                    .name(FK_KEY_BOXES_APPLICATION)
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_KEY_BOXES_APPLICATION)
                    .from(KeyBoxes::Table, KeyBoxes::ApplicationID)
                    .to(Applications::Table, Applications::ID)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(ApplicationSecrets::Table)
                    .name(FK_APPLICATION_SECRET_APPLICATION)
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_APPLICATION_SECRET_APPLICATION)
                    .from(ApplicationSecrets::Table, ApplicationSecrets::ApplicationId)
                    .to(Applications::Table, Applications::ID)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::NoAction)
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
                    .name(FK_USER_APPLICATION)
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_USER_APPLICATION)
                    .from(Users::Table, Users::ApplicationID)
                    .to(Applications::Table, Applications::ID)
                    .on_delete(ForeignKeyAction::NoAction)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(Subjects::Table)
                    .name(FK_SUBJECT_APPLICATION)
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_SUBJECT_APPLICATION)
                    .from(Subjects::Table, Subjects::ApplicationID)
                    .to(Applications::Table, Applications::ID)
                    .on_delete(ForeignKeyAction::NoAction)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(KeyBoxes::Table)
                    .name(FK_KEY_BOXES_APPLICATION)
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_KEY_BOXES_APPLICATION)
                    .from(KeyBoxes::Table, KeyBoxes::ApplicationID)
                    .to(Applications::Table, Applications::ID)
                    .on_delete(ForeignKeyAction::NoAction)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(ApplicationSecrets::Table)
                    .name(FK_APPLICATION_SECRET_APPLICATION)
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name(FK_APPLICATION_SECRET_APPLICATION)
                    .from(ApplicationSecrets::Table, ApplicationSecrets::ApplicationId)
                    .to(Applications::Table, Applications::ID)
                    .on_delete(ForeignKeyAction::NoAction)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Applications {
    Table,
    ID,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    ApplicationID,
}

#[derive(DeriveIden)]
enum Subjects {
    Table,
    ApplicationID,
}

#[derive(DeriveIden)]
enum KeyBoxes {
    Table,
    ApplicationID,
}

#[derive(DeriveIden)]
enum ApplicationSecrets {
    Table,
    ApplicationId,
}
