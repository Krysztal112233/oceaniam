use clap::Parser;
use log::{error, info};
use oceaniam_common::{
    config::{BackendConfig, OceanIamConfig},
    error::Error,
};
use oceaniam_database::{
    helper::{applications::ApplicationHelper, tenants::TenantsHelper},
    model::prelude::*,
};
use sea_orm::TransactionTrait;

use crate::manager::PreManager;

mod manager;

#[derive(Debug, Parser)]
#[command(name = "OceanIAM CLI")]
#[command(bin_name = "oceaniam_cli")]
#[command(arg_required_else_help = true)]
#[command(version)]
#[command(about = "OceanIAM CLI Toolbox")]
enum App {
    /// Initialize all necessary steps for OceanIAM
    Init,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let matches = App::parse();

    match matches {
        App::Init => cmd_init().await,
    }
}

async fn cmd_init() -> Result<(), Error> {
    let config = BackendConfig::new()?;

    let OceanIamConfig {
        application,
        tenant,
    } = &config.oceaniam;

    let PreManager { database, .. } = PreManager::new(&config).await?;

    info!("check if OceanIAM tenant exist...");
    let database = database.begin().await.inspect_err(|e| error!("{e}"))?;
    if !Tenants::is_exist(*tenant, &database)
        .await
        .inspect_err(|e| error!("{e}"))?
    {
        info!("creating OceanIAM tenant...");
        Tenants::create(*tenant, &database)
            .await
            .inspect_err(|e| error!("{e}"))?;
    }
    info!("check if OceanIAM application exist...");
    if !Applications::is_exist(*application, &database)
        .await
        .inspect_err(|e| error!("{e}"))?
    {
        info!("creating OceanIAM application...");
        Applications::create(*application, *tenant, &database)
            .await
            .inspect_err(|e| error!("{e}"))?;
    }
    database.commit().await.inspect_err(|e| error!("{e}"))?;

    Ok(())
}
