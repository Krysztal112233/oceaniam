use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    if let Ok(master_key_hex) = std::env::var("OCEANIAM_MASTER_KEY")
        && let Err(error) = migration::validate_master_key(master_key_hex)
    {
        eprintln!("invalid OCEANIAM_MASTER_KEY: {error}");
        std::process::exit(2);
    }

    cli::run_cli(migration::Migrator).await;
}
