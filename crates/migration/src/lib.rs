pub use sea_orm_migration::prelude::*;

#[allow(unused)]
mod m20220101_000001_create_table;
mod m20260104_172552_add_keybox;
mod m20260105_141454_add_rs384_rs512;
mod m20260105_144550_add_ps256_ps384_ps512;
mod m20260105_150848_alter_key_boxes_kid_key_id;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20260104_172552_add_keybox::Migration),
            Box::new(m20260105_141454_add_rs384_rs512::Migration),
            Box::new(m20260105_144550_add_ps256_ps384_ps512::Migration),
            Box::new(m20260105_150848_alter_key_boxes_kid_key_id::Migration),
        ]
    }
}
