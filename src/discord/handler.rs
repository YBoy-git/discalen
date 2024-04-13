use crate::discord::commands;
use crate::{calendar::Client as CalendarClient, Error};
use serenity::{
    all::{
        Context, CreateInteractionResponse, CreateInteractionResponseMessage, EventHandler, Guild,
        GuildId, Http, Interaction, Ready,
    },
    async_trait,
};
use tracing::{error, info, instrument, warn};

#[derive(Debug)]
pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    #[instrument]
    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: Option<bool>) {
        match is_new {
            None => warn!("Caching is off, can't identify if there are any new servers!"),
            Some(is_new) => {
                if is_new {
                    info!("Added to {} server", guild.name);
                    if let Err(why) = create_calendar(&ctx, guild.name).await {
                        error!(?why, "Failed to create calendar");
                    };
                    init_commands(&ctx.http, &guild.id).await;
                }
            }
        }
    }

    #[instrument]
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let guilds = ready.guilds.iter().map(|guild| &guild.id);
        let mut handles = Vec::with_capacity(guilds.len());
        for guild in guilds {
            handles.push(init_commands(&ctx.http, guild));
        }
        futures::future::join_all(handles).await;
    }

    #[instrument]
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            info!(?command, "Received command interaction");

            let Some(guild_id) = command.guild_id else {
                error!("No guild_id found, cancelling");
                return;
            };
            let channel_id = command.channel_id;
            let options = command.data.options();

            let content = match command.data.name.as_ref() {
                "ping" => Some(commands::ping::run(&options)),
                "create_calendar" => {
                    Some(commands::create_calendar::run(&ctx, guild_id, &options).await)
                }
                "delete_calendar" => {
                    Some(commands::delete_calendar::run(&ctx, guild_id, &options).await)
                }
                "set_event_channel" => Some(
                    commands::set_event_channel::run(&ctx, guild_id, channel_id, &options).await,
                ),
                "list_events" => Some(commands::list_events::run(&ctx, &guild_id, &options).await),
                "create_event" => {
                    Some(commands::create_event::run(&ctx, &guild_id, &options).await)
                }
                "delete_event" => {
                    Some(commands::delete_event::run(&ctx, &guild_id, &options).await)
                }
                command => {
                    error!("An unimplemented command met: {command}");
                    Some("not implemented".to_string())
                }
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new()
                    .content(content)
                    .ephemeral(true);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    error!("Cannot respond to slash command: {why}");
                }
            }
        }
    }
}

#[instrument]
async fn create_calendar(ctx: &Context, name: String) -> Result<(), Error> {
    info!("Pushing a calendar to queue");
    ctx.data
        .read()
        .await
        .get::<CalendarClient>()
        .ok_or(Error::NoCalendarClient)?
        .create_calendar(&name)
        .await?;
    Ok(())
}

async fn init_commands(http: &Http, guild: &GuildId) {
    if let Err(why) = guild
        .set_commands(
            http,
            vec![
                commands::ping::register(),
                commands::create_calendar::register(),
                commands::delete_calendar::register(),
                commands::set_event_channel::register(),
                commands::list_events::register(),
                commands::create_event::register(),
                commands::delete_event::register(),
            ],
        )
        .await
    {
        error!(?why, "Failed to create a command");
    };
}
