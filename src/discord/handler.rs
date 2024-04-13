use crate::discord::commands;
use serenity::{
    all::{
        Context, CreateInteractionResponse, CreateInteractionResponseMessage, EventHandler, Guild,
        Interaction, Message, Ready,
    },
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

        // TODO remove when no more needed
        if content == "!discalen create_calendar" {
            create_calendar(&ctx, msg.guild_id.unwrap().to_string()).await;
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
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let guilds = ready.guilds.into_iter().map(|guild| guild.id);
        for guild in guilds {
            guild
                .set_commands(
                    &ctx.http,
                    vec![
                        commands::ping::register(),
                        commands::set_event_channel::register(),
                        commands::list_events::register(),
                        commands::create_event::register(),
                        commands::delete_event::register(),
                    ],
                )
                .await
                .unwrap();
        }
    }

    #[instrument]
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            info!(?command, "Received command interaction");

            let content = match command.data.name.as_ref() {
                "ping" => Some(commands::ping::run(&command.data.options())),
                "set_event_channel" => Some(
                    commands::set_event_channel::run(
                        &ctx,
                        command.guild_id.unwrap(),
                        command.channel_id,
                        &command.data.options(),
                    )
                    .await,
                ),
                "list_events" => Some(
                    commands::list_events::run(
                        &ctx,
                        &command.guild_id.unwrap(),
                        &command.data.options(),
                    )
                    .await,
                ),
                "create_event" => Some(
                    commands::create_event::run(
                        &ctx,
                        &command.guild_id.unwrap(),
                        &command.data.options(),
                    )
                    .await,
                ),
                "delete_event" => Some(
                    commands::delete_event::run(
                        &ctx,
                        &command.guild_id.unwrap(),
                        &command.data.options(),
                    )
                    .await,
                ),
                command => {
                    error!("An unimplemented command met: {command}");
                    Some("not implemented".to_string())
                }
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    error!("Cannot respond to slash command: {why}");
                }
            }
        }
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
