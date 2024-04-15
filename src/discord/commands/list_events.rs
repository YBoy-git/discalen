use crate::{calendar::Client as CalendarClient, Error};
use serenity::all::{Context, CreateCommand, GuildId, ResolvedOption};
use tracing::{info, instrument, warn};

use crate::calendar::get_calendar_url;

use super::MessageResult;

#[instrument]
pub async fn run(
    ctx: &Context,
    guild_id: &GuildId,
    _options: &[ResolvedOption<'_>],
) -> MessageResult {
    info!("Fetching an event list for a guild");
    let lock = ctx.data.read().await;
    let calendar_client = lock
        .get::<CalendarClient>()
        .ok_or(Error::NoCalendarClient)?;

    let calendars = calendar_client.get_calendars_by_guild_id(guild_id).await?;

    let calendar = match calendars {
        Some(calendar) => calendar.id.expect("No calendar id"),
        None => {
            warn!("No calendar found for the server, escaping...");
            return Ok(
                "No calendar found for the server! Create a new one using `/create_calendar`"
                    .into(),
            );
        }
    };

    let events = calendar_client.list_events(&calendar).await?;

    info!(?events, "Returned event list");

    Ok(format!(
        "Events:\n{}\nCalendar: {}",
        events
            .into_iter()
            .map(|event| {
                let label = event.summary.unwrap_or_else(|| {
                    warn!(event_id = event.id, "No label for the event");
                    "No label".into()
                });
                let date = match event.start {
                    Some(start) => match start.date {
                        Some(date) => date.to_string(),
                        None => {
                            warn!(event_id = event.id, "No date for the event");
                            "No date".into()
                        }
                    }
                    .to_string(),
                    None => {
                        warn!(event_id = event.id, "No start for the event");
                        "No start".into()
                    }
                };
                format!("{}: {}", label, date)
            })
            .collect::<Vec<_>>()
            .join("\n"),
        get_calendar_url(&calendar)
    ))
}

pub fn register() -> CreateCommand {
    CreateCommand::new("list_events").description("List all the events on the server")
}
