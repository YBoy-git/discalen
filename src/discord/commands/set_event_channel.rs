use serenity::all::{ChannelId, Context, GuildId, Permissions};
use serenity::builder::CreateCommand;
use serenity::model::application::ResolvedOption;
use tracing::{info, instrument};

use crate::discord::set_event_channel;

#[instrument]
pub async fn run(
    ctx: &Context,
    guild_id: GuildId,
    channel_id: ChannelId,
    _options: &[ResolvedOption<'_>],
) -> String {
    info!("Setting the event channel");
    let lock = ctx.data.read().await;
    let pool = lock.get::<crate::Pool>().expect("No DB pool found");
    set_event_channel(pool, &guild_id, &channel_id).await;
    "The channel is set as the event channel for the server!".into()
}

pub fn register() -> CreateCommand {
    CreateCommand::new("set_event_channel")
        .description("The channel will be receiving notifications about any current events")
        .default_member_permissions(Permissions::ADMINISTRATOR)
}
