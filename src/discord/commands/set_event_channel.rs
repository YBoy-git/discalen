use serenity::all::{ChannelId, Context, GuildId};
use serenity::builder::CreateCommand;
use serenity::model::application::ResolvedOption;
use tracing::{info, instrument};

#[instrument]
pub async fn run(
    ctx: &Context,
    guild_id: GuildId,
    channel_id: ChannelId,
    _options: &[ResolvedOption<'_>],
) -> String {
    info!("Setting the event channel");
    ctx.data
        .write()
        .await
        .get_mut::<crate::discord::Data>()
        .unwrap()
        .event_channels
        .insert(guild_id, channel_id);
    "The channel is set as the event channel for the server!".into()
}

pub fn register() -> CreateCommand {
    CreateCommand::new("set_event_channel")
        .description("The channel will be receiving notifications about any current events")
}
