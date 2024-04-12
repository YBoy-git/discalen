use std::str::FromStr;

use chrono::{Datelike, Days};
use google_calendar3::api::{Event, EventDateTime};
use serenity::{
    all::{ChannelId, Context, CreateMessage, EventHandler, Guild, GuildId, Message, Ready},
    async_trait,
};
use tracing::{error, info, instrument, warn};

#[derive(Debug)]
pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    #[instrument]
    async fn message(&self, ctx: Context, msg: Message) {
        let content = msg.content;

        if content == "!discalen health_check" {
            info!("Health checked");
            if let Err(why) = msg
                .channel_id
                .say(&ctx.http, format!("Hey {}!", msg.author.name))
                .await
            {
                error!("Error sending message: {why:?}");
            }
        }
        if content == "!discalen set_event_channel" {
            match msg.guild_id {
                Some(guild_id) => {
                    set_event_channel(&ctx, guild_id, msg.channel_id).await;
                }
                None => error!("No guild_id found"),
            };
        }
        if content == "!discalen create_calendar" {
            create_calendar(&ctx, msg.guild_id.unwrap().to_string()).await;
        }
        if content == "!discalen list_events" {
            list_events(&ctx, msg.guild_id.unwrap()).await;
        }
        // bad parsing, any serenity features? if not, clap?
        if let Some(event_info) = content.strip_prefix("!discalen create_event") {
            let event_info = event_info.trim();
            let mut parts = event_info.split(' ');
            let label = parts.next().unwrap();
            let date = parts.next().unwrap();
            create_event(&ctx, label, date, msg.guild_id.unwrap()).await;
        }
        if let Some(label) = content.strip_prefix("!discalen delete_event") {
            let label = label.trim();
            delete_event(&ctx, label, msg.guild_id.unwrap()).await;
        }
    }

    #[instrument]
    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: Option<bool>) {
        match is_new {
            None => warn!("Caching is off, can't identify if there are any new servers!"),
            Some(is_new) => {
                if is_new {
                    info!("Added to {} server", guild.name);
                    create_calendar(&ctx, guild.name).await;
                }
            }
        }
    }

    #[instrument]
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

#[instrument]
async fn set_event_channel(ctx: &Context, guild_id: GuildId, channel_id: ChannelId) {
    info!("Setting event channel");
    ctx.data
        .write()
        .await
        .get_mut::<crate::discord::Data>()
        .unwrap()
        .event_channels
        .insert(guild_id, channel_id);
    if let Err(why) = channel_id
        .say(&ctx.http, "Set as the event channel!".to_string())
        .await
    {
        error!("Error sending message: {why:?}");
    }
}

#[instrument]
async fn create_calendar(ctx: &Context, name: String) {
    info!("Pushing a calendar to queue");
    ctx.data
        .write()
        .await
        .get_mut::<crate::calendar::Client>()
        .unwrap()
        .create_calendar(&name)
        .await;
}

#[instrument]
async fn list_events(ctx: &Context, guild_id: GuildId) {
    // info!(?event, "Creating an event");
    let lock = ctx.data.read().await;
    let calendar_client = lock.get::<crate::calendar::Client>().unwrap();
    let discord_data = lock.get::<crate::discord::Data>().unwrap();

    let events = calendar_client
        .list_events(
            &calendar_client
                .get_calendar_id_by_guild_id(guild_id)
                .await
                .unwrap(),
        )
        .await;

    info!(?events, "Returned event list");

    let message = CreateMessage::new().content(format!(
        "Events: {}",
        events
            .into_iter()
            .map(|event| {
                format!(
                    "{}: {}",
                    event.summary.unwrap(),
                    event.start.unwrap().date.unwrap()
                )
            })
            .collect::<Vec<_>>()
            .join("; ")
    ));
    ctx.http
        .send_message(
            *discord_data.event_channels.get(&guild_id).unwrap(),
            vec![],
            &message,
        )
        .await
        .unwrap();
}

#[instrument]
async fn create_event(ctx: &Context, label: &str, date: &str, guild_id: GuildId) {
    info!("Pushing an event to the queue");
    let year = chrono::Utc::now().year();
    let date = format!("{year}-{date}");
    let date = chrono::NaiveDate::from_str(&date).unwrap();
    let event = Event {
        summary: Some(label.to_string()),
        start: Some(EventDateTime {
            date: Some(date),
            ..Default::default()
        }),
        end: Some(EventDateTime {
            date: Some(date.checked_add_days(Days::new(1)).unwrap()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let lock = ctx.data.read().await;
    let calendar_client = lock.get::<crate::calendar::Client>().unwrap();
    calendar_client
        .calendar_hub
        .events()
        .insert(
            event,
            &calendar_client
                .get_calendar_id_by_guild_id(guild_id)
                .await
                .unwrap(),
        )
        .doit()
        .await
        .unwrap();
}

#[instrument]
async fn delete_event(ctx: &Context, label: &str, guild_id: GuildId) {
    let lock = ctx.data.read().await;
    let calendar_client = lock.get::<crate::calendar::Client>().unwrap();
    let event_ids = calendar_client.get_event_id_by_label(label, guild_id).await;
    info!(?event_ids, "Deleting these events");
    for id in event_ids {
        calendar_client
            .calendar_hub
            .events()
            .delete(
                &calendar_client
                    .get_calendar_id_by_guild_id(guild_id)
                    .await
                    .unwrap(),
                &id,
            )
            .doit()
            .await
            .unwrap();
    }
}
