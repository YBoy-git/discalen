use std::sync::Arc;

use calendar::Client as CalendarClient;
use chrono::Utc;
use config::AppConfig;
use discord::Client as DiscordClient;
use discord::Handler;
use futures::StreamExt;
use google_calendar3::api::CalendarListEntry;
use google_calendar3::api::Event;
use google_calendar3::api::EventDateTime;
use secrecy::ExposeSecret;
use serenity::all::CreateMessage;
use serenity::all::GuildId;
use serenity::all::Http;
use serenity::prelude::*;
use serenity::Client as SerenityClient;
use sqlx::PgPool;
use tokio::task::JoinHandle;
use tracing::error;
use tracing::instrument;
use tracing::warn;

pub mod config;

mod calendar;
mod discord;

mod error;
pub use error::*;

pub struct Pool;

impl TypeMapKey for Pool {
    type Value = PgPool;
}

pub struct Client {
    discord_client: DiscordClient,
}

impl Client {
    pub async fn run(config: AppConfig, pool: PgPool) -> Result<(), Error> {
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILDS;

        let calendar_client =
            CalendarClient::with_sa_key(config.google_secret.expose_secret()).await?;

        let serenity_client =
            SerenityClient::builder(config.discord_access_token.expose_secret(), intents)
                .event_handler(Handler)
                .await?;
        let serenity_data = serenity_client.data.clone();
        {
            let mut data = serenity_data.write().await;
            data.insert::<CalendarClient>(calendar_client.clone());
            data.insert::<Pool>(pool);
        }

        let discalen_client = Self {
            discord_client: DiscordClient::new(serenity_client).await,
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

        let calendar_task: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
            let calendar_client = calendar_client;
            loop {
                tokio::time::sleep(config.notification_period).await;

                let calendars = calendar_client.list_calendars().await?;

                let mut calendars_handles = vec![];
                for calendar in &calendars {
                    let handle = calendar_client
                        .list_events(calendar.id.as_ref().unwrap_or_else(|| unreachable!()));
                    calendars_handles.push((calendar, handle));
                }

                let mut stream = futures::stream::iter(calendars_handles);
                while let Some(handle) = stream.next().await {
                    let (calendar, events) = handle;
                    let events = events.await?;

                    let mut sending_tasks = vec![];
                    for event in events {
                        let date = match event.start {
                            Some(EventDateTime {
                                date: Some(date), ..
                            }) => date,
                            _ => {
                                warn!(?event, "The event doesn't have the start date, skipping...");
                                continue;
                            }
                        };
                        if date == Utc::now().date_naive() {
                            sending_tasks.push(send_event_notification(
                                discord_data.clone(),
                                &sender_http,
                                calendar,
                                event,
                            ));
                        }
                    }
                    futures::future::join_all(sending_tasks).await;
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

#[instrument(skip(data))]
async fn send_event_notification(
    data: Arc<RwLock<TypeMap>>,
    sender_http: &Http,
    calendar: &CalendarListEntry,
    event: Event,
) -> Result<(), Error> {
    let lock = data.read().await;
    let summary = calendar.summary.as_ref().unwrap_or_else(|| unreachable!());
    let guild_id = match summary.parse() {
        Ok(guild_id) => guild_id,
        Err(why) => {
            error!(?why, "Failed to parse calendar summary to guild id");
            return Err(Error::CalendarSummaryNotDiscordServerId(summary.into()));
        }
    };
    let guild_id = GuildId::new(guild_id);
    let pool = lock.get::<Pool>().expect("No pool found, exiting");
    let Some(channel_id) = discord::get_event_channel_id(pool, &guild_id).await? else {
        warn!(?guild_id, "The server has no event channel");
        return Err(Error::DiscordSeverHasNoEventChannel(guild_id));
    };
    let message = CreateMessage::new().content(format!(
        "Today is {}, have a nice celebration!🎉",
        match event.summary.as_ref() {
            Some(summary) => summary.as_str(),
            None => "No label",
        }
    ));
    if let Err(why) = sender_http.send_message(channel_id, vec![], &message).await {
        error!(?why, "Failed to send an event notification")
    };
    Ok(())
}
