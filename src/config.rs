use std::{fs, path::Path, time::Duration};

use config::Config;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

use crate::Error;

#[derive(Deserialize)]
pub struct AppConfig {
    #[serde(with = "humantime_serde")]
    pub notification_period: Duration,
    pub discord_access_token: Secret<String>,
    pub google_secret: Secret<String>,
    pub db: DbConfig,
}

#[derive(Deserialize)]
pub struct DbConfig {
    pub user: String,
    pub password: Secret<String>,
    pub host: String,
    pub port: u16,
    pub name: String,
}

impl DbConfig {
    pub fn get_connection_url(&self) -> Secret<String> {
        format!(
            "postgres://{}:{}@{}:{}",
            self.user,
            self.password.expose_secret(),
            self.host,
            self.port
        )
        .into()
    }

    pub fn get_database_url(&self) -> Secret<String> {
        format!(
            "{}/{}",
            self.get_connection_url().expose_secret(),
            self.name
        )
        .into()
    }
}

impl AppConfig {
    pub fn load(secrets_path: impl AsRef<Path>) -> Result<Self, Error> {
        let secrets_path = secrets_path.as_ref();

        let config = Config::builder();
        let config = config.add_source(config::File::with_name("config"));
        let discord_token = fs::read_to_string(secrets_path.join("discord-token.txt"))?;
        let config = config.set_override("discord_access_token", discord_token)?;
        let google_secret = fs::read_to_string(secrets_path.join("google-sa-secret.json"))?;
        let config = config.set_override("google_secret", google_secret)?;
        let db_password = fs::read_to_string(secrets_path.join("db_password.txt"))?;
        let config = config.set_override("db.password", db_password)?;
        let config = config.build()?;

        config.try_deserialize().map_err(Into::into)
    }
}
