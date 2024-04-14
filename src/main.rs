use std::env;

use anyhow::{Context, Result};
use discalen::config::AppConfig;
use secrecy::ExposeSecret;
use sqlx::PgPool;
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
    let config = AppConfig::load(secrets_path).context("Failed to init the config")?;
    let pool = PgPool::connect(config.db.get_database_url().expose_secret())
        .await
        .context("Failed to connect to db")?;

    discalen::Client::run(config, pool).await?;

    Ok(())
}
