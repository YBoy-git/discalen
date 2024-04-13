use std::env;

use anyhow::{Context, Result};
use discalen::config::AppConfig;
use tracing::instrument;

#[instrument]
#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt().init();

    let secrets_path = format!(
        "{}/secrets",
        env::var("CARGO_MANIFEST_DIR").context("Couldn't read CARGO_MANIFEST_DIR")?
    );
    let config = AppConfig::load(secrets_path);

    discalen::Client::run(config).await?;

    Ok(())
}
