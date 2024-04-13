use std::{fs, path::Path, time::Duration};

use config::Config;
use secrecy::Secret;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AppConfig {
    #[serde(with = "humantime_serde")]
    pub notification_period: Duration,
    pub discord_access_token: Secret<String>,
    pub google_secret: Secret<String>,
}

impl AppConfig {
    pub fn load(secrets_path: impl AsRef<Path>) -> Self {
        let secrets_path = secrets_path.as_ref();

        let config = Config::builder();
        let config = config.add_source(config::File::with_name("config"));
        let discord_token =
            fs::read_to_string(secrets_path.join("discord-token.txt")).unwrap();
        let config = config
            .set_override("discord_access_token", discord_token)
            .unwrap();
        let google_secret =
            fs::read_to_string(secrets_path.join("google-sa-secret.json")).unwrap();
        let config = config.set_override("google_secret", google_secret).unwrap();
        let config = config.build().unwrap();

        config.try_deserialize().unwrap()
    }
}
