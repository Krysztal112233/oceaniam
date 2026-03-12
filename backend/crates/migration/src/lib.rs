pub use sea_orm_migration::prelude::*;

#[allow(unused)]
mod m20220101_000001_create_table;
mod m20260104_172552_add_keybox;
mod m20260105_141454_add_rs384_rs512;
mod m20260105_144550_add_ps256_ps384_ps512;
mod m20260105_150848_alter_key_boxes_kid_key_id;
mod m20260106_090640_create_tenant_application;
mod m20260106_165217_alter_subject_application_id;
mod m20260107_060743_alter_application_comment_nullable;
mod m20260107_082109_alter_subject_id;
mod m20260107_084436_fk_user_id_subject_id;
mod m20260109_155812_alter_tenants_comment_nullable;
mod m20260201_105619_create_basic_management;
mod m20260206_092132_create_revoked_jwts;
mod m20260206_095214_alter_key_boxes_belong_to;
mod m20260207_124924_alter_key_boxes_key_id_type;
mod m20260207_160434_alter_drop_key_boxes_key_id;
mod m20260217_161452_alter_rename_applications_tenants_id;
mod m20260217_174210_alter_users_email;
mod m20260217_175203_alter_credentials_rename_subject_id_id;
mod m20260217_181210_alter_credentials_value;
mod m20260218_164739_move_adminstrators_phc_credentials_phc;
mod m20260218_180745_alter_fk_credential_subjects_adminstrators;
mod m20260222_103403_create_application_secret;
mod m20260223_084124_create_nickname_for_users;
mod m20260302_163002_alter_application_configs;
mod m20260303_095046_alter_fill_default_configuration;
mod m20260311_100109_alter_configuration_empty_audience;
mod m20260311_170219_create_audit_table;

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
            Box::new(m20260106_090640_create_tenant_application::Migration),
            Box::new(m20260106_165217_alter_subject_application_id::Migration),
            Box::new(m20260107_060743_alter_application_comment_nullable::Migration),
            Box::new(m20260107_082109_alter_subject_id::Migration),
            Box::new(m20260107_084436_fk_user_id_subject_id::Migration),
            Box::new(m20260109_155812_alter_tenants_comment_nullable::Migration),
            Box::new(m20260201_105619_create_basic_management::Migration),
            Box::new(m20260206_092132_create_revoked_jwts::Migration),
            Box::new(m20260206_095214_alter_key_boxes_belong_to::Migration),
            Box::new(m20260207_124924_alter_key_boxes_key_id_type::Migration),
            Box::new(m20260207_160434_alter_drop_key_boxes_key_id::Migration),
            Box::new(m20260217_161452_alter_rename_applications_tenants_id::Migration),
            Box::new(m20260217_174210_alter_users_email::Migration),
            Box::new(m20260217_175203_alter_credentials_rename_subject_id_id::Migration),
            Box::new(m20260217_181210_alter_credentials_value::Migration),
            Box::new(m20260218_164739_move_adminstrators_phc_credentials_phc::Migration),
            Box::new(m20260218_180745_alter_fk_credential_subjects_adminstrators::Migration),
            Box::new(m20260222_103403_create_application_secret::Migration),
            Box::new(m20260223_084124_create_nickname_for_users::Migration),
            Box::new(m20260302_163002_alter_application_configs::Migration),
            Box::new(m20260303_095046_alter_fill_default_configuration::Migration),
            Box::new(m20260311_100109_alter_configuration_empty_audience::Migration),
            Box::new(m20260311_170219_create_audit_table::Migration),
        ]
    }
}
