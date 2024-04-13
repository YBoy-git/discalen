use crate::{calendar::Client as CalendarClient, Error};
use serenity::all::{Context, CreateCommand, GuildId, Permissions, ResolvedOption};
use tracing::{info, instrument};

use crate::calendar::get_calendar_url;

#[instrument]
pub async fn run(
    ctx: &Context,
    guild_id: GuildId,
    _options: &[ResolvedOption<'_>],
) -> Result<String, Error> {
    info!("Creating a calendar");
    let lock = ctx.data.read().await;
    let calendar_client = lock
        .get::<CalendarClient>()
        .ok_or(Error::NoCalendarClient)?;
    let calendars = calendar_client.get_calendars_by_guild_id(&guild_id).await?;
    if calendars.is_some() {
        return Ok(format!(
            "A calendar already exists: {}",
            calendars
                .into_iter()
                .map(|calendar| {
                    let calendar_id = calendar.id.expect("No calendar id");
                    get_calendar_url(&calendar_id)
                })
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    calendar_client
        .create_calendar(&guild_id.to_string())
        .await?;
    Ok("Created the calendar! `/list_events` to get the url".into())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("create_calendar")
        .description("Create a new event calendar for the server, removing the old")
        .default_member_permissions(Permissions::ADMINISTRATOR)
}
