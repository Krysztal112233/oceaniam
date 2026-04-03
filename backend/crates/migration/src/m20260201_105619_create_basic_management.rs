use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use rand::{Rng, rngs::OsRng};
use sea_orm::{ActiveModelTrait, ActiveValue::Set};
use sea_orm_migration::{prelude::*, schema::*};
use uuid::Uuid;

#[derive(DeriveMigrationName)]
pub struct Migration;

const DEFAULT_ROOT_PASSWORD_ENV: &str = "MIGRATION_DEFAULT_ROOT_PASSWORD";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Administrators::Table)
                    .col(pk_uuid(Administrators::ID))
                    .col(string_uniq(Administrators::Name))
                    .col(string(Administrators::Phc))
                    .to_owned(),
            )
            .await?;

        let (password, phc) = {
            let password = initial_root_password();
            let salt = SaltString::generate(&mut OsRng);

            (
                password.clone(),
                Argon2::default()
                    .hash_password(password.as_bytes(), &salt)
                    .unwrap()
                    .to_string(),
            )
        };

        println!("Initialling basic root account");
        generated::ActiveModel {
            id: Set(Uuid::now_v7()),
            name: Set("root".into()),
            phc: Set(phc.clone()),
        }
        .insert(manager.get_connection())
        .await?;

        println!("====THIS INFORMATION ONLY SHOW ONCE====");
        println!("account: root");
        println!("password: {password}");
        println!("=======================================");

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Administrators::Table).to_owned())
            .await?;
        Ok(())
    }
}

fn initial_root_password() -> String {
    match std::env::var(DEFAULT_ROOT_PASSWORD_ENV) {
        Ok(password) if password.len() > 6 => password,
        _ => gen_password(),
    }
}

fn gen_password() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";
    const PASSWORD_LEN: usize = 16;
    let mut rng = rand::thread_rng();

    (0..PASSWORD_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[derive(DeriveIden)]
enum Administrators {
    Table,

    ID,
    Name,
    Phc,
}

mod generated {
    use crate::sea_orm::{DerivePrimaryKey, PrimaryKeyTrait};

    use sea_orm::{ActiveModelBehavior, DeriveEntityModel, DeriveRelation, EnumIter};
    use sea_orm_migration::prelude::*;
    use uuid::Uuid;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
    #[sea_orm(table_name = "administrators")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub id: Uuid,
        #[sea_orm(unique)]
        pub name: String,
        pub phc: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
