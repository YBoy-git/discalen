use crate::{calendar::Client as CalendarClient, Error};
use serenity::all::{Context, CreateCommand, GuildId, Permissions, ResolvedOption};
use tracing::{info, instrument};

use super::MessageResult;

#[instrument]
pub async fn run(
    ctx: &Context,
    guild_id: GuildId,
    _options: &[ResolvedOption<'_>],
) -> MessageResult {
    info!("Deleting calendars");
    let lock = ctx.data.read().await;
    let calendar_client = lock
        .get::<CalendarClient>()
        .ok_or(Error::NoCalendarClient)?;
    let calendars = calendar_client.get_calendars_by_guild_id(&guild_id).await?;

    let mut handles = vec![];
    for calendar in &calendars {
        let calendar_id = calendar.id.as_ref().expect("No calendar id");
        let handle = calendar_client.delete_calendar(calendar_id);
        handles.push(handle);
    }
    futures::future::join_all(handles).await;

    Ok("Deleted calendars!".into())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("delete_calendar")
        .description("Delete the event calendar associated with the server")
        .default_member_permissions(Permissions::ADMINISTRATOR)
}
