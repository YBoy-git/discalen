use std::time::Duration;

use anyhow::{Context, Result};
use chrono::Utc;
use discord::Handler;
use once_cell::sync::Lazy;
use serenity::prelude::*;
use serenity::Client as SerenityClient;
use tracing::info;

mod calendar;
mod discord;

static SLEEP_TIME: Lazy<Duration> = Lazy::new(|| Duration::from_secs(5));

pub struct Client {
    discord_client: discord::Client,
}

impl Client {
    pub async fn run(discord_token: String) -> Result<()> {
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILDS;

        let calendar_client = calendar::Client::default().await;

        let serenity_client = SerenityClient::builder(&discord_token, intents)
            .event_handler(Handler)
            .await
            .context("Err creating client")?;
        let serenity_data = serenity_client.data.clone();
        {
            let mut data = serenity_data.write().await;
            data.insert::<calendar::Client>(calendar_client.clone());
        }

        let discalen_client = Self {
            discord_client: discord::Client::new(serenity_client).await,
        };
        let discord_client = discalen_client.discord_client;
        let sender_http = discord_client.serenity_client.http.clone();

        let discord_task = tokio::spawn(async move {
            let mut serenity_client = discord_client.serenity_client;
            if let Err(why) = serenity_client.start_autosharded().await {
                println!("Client error: {why:?}");
            }
        });

        let calendar_task = tokio::spawn(async move {
            let calendar_client = calendar_client;
            loop {
                tokio::time::sleep(*SLEEP_TIME).await;

                let calendars = calendar_client.list_calendars().await;
                for calendar in calendars {
                    let events = calendar_client.list_events(&calendar).await;
                    for event in events {
                        let date = event.start.unwrap().date.unwrap();
                        if date == Utc::now().date_naive() {
                            info!("There's an event today!");
                        }
                    }
                }
            }
        });

        tokio::select! {
            _ = discord_task => (),
            _ = calendar_task => (),
        };

        Ok(())
    }
}
