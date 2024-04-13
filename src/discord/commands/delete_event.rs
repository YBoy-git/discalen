use crate::{calendar::Client as CalendarClient, Error};
use serenity::all::{
    CommandOptionType, Context, CreateCommand, CreateCommandOption, GuildId, Permissions,
    ResolvedOption, ResolvedValue,
};
use tracing::{info, instrument, warn};

use super::MessageResult;

#[instrument]
pub async fn run(
    ctx: &Context,
    guild_id: &GuildId,
    options: &[ResolvedOption<'_>],
) -> MessageResult {
    let Some(ResolvedOption {
        value: ResolvedValue::String(label),
        ..
    }) = options.first()
    else {
        return Err(Error::MissingParameter("label".into()));
    };

    let lock = ctx.data.read().await;
    let calendar_client = lock
        .get::<CalendarClient>()
        .ok_or(Error::NoCalendarClient)?;
    let calendars = calendar_client.get_calendars_by_guild_id(guild_id).await?;
    let Some(calendar) = calendars else {
        warn!("Couldn't find a calendar for the guild");
        return Ok("No calendar for the server, create a new one! `/create_calendar`".into());
    };
    let calendar_id = calendar.id.expect("No calendar id");

    let event_ids = calendar_client
        .get_event_id_by_label(label, &calendar_id)
        .await?;
    info!(?event_ids, "Deleting these events");

    let mut handles = Vec::with_capacity(event_ids.len());
    for id in &event_ids {
        let handle = calendar_client.delete_event(id, &calendar_id);
        handles.push(handle);
    }
    futures::future::join_all(handles).await;

    Ok(format!(
        "Deleted {} events from the calendar!",
        event_ids.len()
    ))
}

pub fn register() -> CreateCommand {
    CreateCommand::new("delete_event")
        .description("Delete events with the specified label")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "label", "The label of the event")
                .required(true),
        )
        .default_member_permissions(Permissions::ADMINISTRATOR)
}
