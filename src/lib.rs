use anyhow::{Context, Result};
use chrono::Utc;
use config::AppConfig;
use discord::Handler;
use secrecy::ExposeSecret;
use serenity::all::CreateMessage;
use serenity::all::GuildId;
use serenity::prelude::*;
use serenity::Client as SerenityClient;

pub mod config;

mod calendar;
mod discord;

pub struct Client {
    discord_client: discord::Client,
}

impl Client {
    pub async fn run(config: AppConfig) -> Result<()> {
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILDS;

        let calendar_client =
            calendar::Client::with_sa_key(config.google_secret.expose_secret()).await;

        let serenity_client =
            SerenityClient::builder(config.discord_access_token.expose_secret(), intents)
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
        let discord_data = discord_client.serenity_client.data.clone();

        let discord_task = tokio::spawn(async move {
            let mut serenity_client = discord_client.serenity_client;
            if let Err(why) = serenity_client.start_autosharded().await {
                println!("Client error: {why:?}");
            }
        });

        let calendar_task = tokio::spawn(async move {
            let calendar_client = calendar_client;
            loop {
                tokio::time::sleep(config.notification_period).await;

                let calendars = calendar_client.list_calendars().await;
                for calendar in calendars {
                    let events = calendar_client
                        .list_events(calendar.id.as_ref().unwrap())
                        .await;
                    for event in events {
                        let date = event.start.unwrap().date.unwrap();
                        if date == Utc::now().date_naive() {
                            let lock = discord_data.read().await;
                            let channel_id = lock
                                .get::<discord::Data>()
                                .unwrap()
                                .event_channels
                                .get(&GuildId::new(
                                    calendar.summary.as_ref().unwrap().parse().unwrap(),
                                ))
                                .unwrap();
                            let message = CreateMessage::new().content(format!(
                                "Today is {}, have a nice celebration!🎉",
                                event.summary.unwrap()
                            ));
                            sender_http
                                .send_message(*channel_id, vec![], &message)
                                .await
                                .unwrap();
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
