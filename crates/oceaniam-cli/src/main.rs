use clap::Parser;
use oceaniam_common::{config::BackendConfig, error::Error};

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

    let config = BackendConfig::new()?;

    let manager = PreManager::new(config).await?;

    Ok(())
}
