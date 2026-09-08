use sea_orm::{DatabaseBackend, Statement};
use sea_orm_migration::{prelude::*, schema::*};

const UQ_USERS_OIDC_SUB: &str = "uq_users_oidc_sub";

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260907_174633_add_users_oidc_sub"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if !manager.has_column("users", "oidc_sub").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(Users::Table)
                        .add_column(uuid_null(Users::OidcSub))
                        .to_owned(),
                )
                .await?;
        }

        ensure_oidc_sub_column_is_compatible(manager).await?;

        manager
            .execute(
                Query::update()
                    .table(Users::Table)
                    .value(Users::OidcSub, Expr::col(Users::Id))
                    .and_where(Expr::col(Users::OidcSub).is_null())
                    .to_owned(),
            )
            .await?;

        if manager.has_index("users", UQ_USERS_OIDC_SUB).await? {
            ensure_oidc_sub_index_is_compatible(manager).await?;
        } else {
            manager
                .create_index(
                    Index::create()
                        .name(UQ_USERS_OIDC_SUB)
                        .table(Users::Table)
                        .col(Users::OidcSub)
                        .unique()
                        .to_owned(),
                )
                .await?;
        }

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .modify_column(ColumnDef::new(Users::OidcSub).uuid().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if manager.has_index("users", UQ_USERS_OIDC_SUB).await? {
            manager
                .drop_index(
                    Index::drop()
                        .name(UQ_USERS_OIDC_SUB)
                        .table(Users::Table)
                        .to_owned(),
                )
                .await?;
        }

        if manager.has_column("users", "oidc_sub").await? {
            manager
                .alter_table(
                    Table::alter()
                        .table(Users::Table)
                        .drop_column(Users::OidcSub)
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }
}

async fn ensure_oidc_sub_column_is_compatible(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let column = manager
        .get_connection()
        .query_one_raw(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT data_type, column_default FROM information_schema.columns \
             WHERE table_schema = current_schema() AND table_name = 'users' \
             AND column_name = 'oidc_sub'"
                .to_owned(),
        ))
        .await?
        .ok_or_else(|| DbErr::Custom("users.oidc_sub was not created".to_owned()))?;

    let data_type: String = column.try_get("", "data_type")?;
    let column_default: Option<String> = column.try_get("", "column_default")?;
    if data_type != "uuid" || column_default.is_some() {
        return Err(DbErr::Custom(
            "users.oidc_sub must be UUID with no database default".to_owned(),
        ));
    }

    Ok(())
}

async fn ensure_oidc_sub_index_is_compatible(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let index = manager
        .get_connection()
        .query_one_raw(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"SELECT i.indisunique
                       AND i.indisvalid
                       AND i.indisready
                       AND i.indpred IS NULL
                       AND i.indnatts = 1
                       AND attribute.attname = 'oidc_sub' AS valid
                FROM pg_class index
                JOIN pg_index i ON i.indexrelid = index.oid
                JOIN pg_class users ON users.oid = i.indrelid
                JOIN pg_namespace namespace ON namespace.oid = users.relnamespace
                JOIN pg_attribute attribute
                  ON attribute.attrelid = users.oid
                 AND attribute.attnum = i.indkey[0]
                WHERE namespace.nspname = current_schema()
                  AND users.relname = 'users'
                  AND index.relname = $1"#,
            [UQ_USERS_OIDC_SUB.into()],
        ))
        .await?
        .ok_or_else(|| DbErr::Custom(format!("index {UQ_USERS_OIDC_SUB} was not found")))?;

    if !index.try_get::<bool>("", "valid")? {
        return Err(DbErr::Custom(format!(
            "index {UQ_USERS_OIDC_SUB} must uniquely cover only users.oidc_sub"
        )));
    }

    Ok(())
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    OidcSub,
}
