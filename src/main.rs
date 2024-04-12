use std::env;

use anyhow::{Context, Result};
use tracing::instrument;

#[instrument]
#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt().init();

    let token = env::var("DISCORD_TOKEN").context("Expected a token in the environment")?;
    discalen::Client::run(token).await?;

    Ok(())
}
