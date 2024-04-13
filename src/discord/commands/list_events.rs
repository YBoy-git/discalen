use crate::calendar::Client as CalendarClient;
use serenity::all::{Context, CreateCommand, GuildId, ResolvedOption};
use tracing::{error, info, instrument, warn};

use crate::calendar::get_calendar_url;

#[instrument]
pub async fn run(ctx: &Context, guild_id: &GuildId, _options: &[ResolvedOption<'_>]) -> String {
    info!("Fetching an event list for a guild");
    let lock = ctx.data.read().await;
    let calendar_client = lock
        .get::<CalendarClient>()
        .expect("No calendar client found");

    let mut calendars = match calendar_client.get_calendars_by_guild_id(guild_id).await {
        Ok(calendars) => calendars,
        Err(why) => {
            error!(?why, "Failed to get calendars by guild id");
            return format!("An error occurred: {why}");
        }
    };

    let calendar = match calendars.pop() {
        Some(calendar) => calendar.id.unwrap_or_else(|| unreachable!()),
        None => {
            return "No calendar found for the server! Create a new one using `/create_calendar`"
                .into()
        }
    };

    let events = match calendar_client.list_events(&calendar).await {
        Ok(events) => events,
        Err(why) => {
            error!(?why, "Failed to list events");
            return format!("An error occurred: {why}");
        }
    };

    info!(?events, "Returned event list");

    format!(
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
    )
}

pub fn register() -> CreateCommand {
    CreateCommand::new("list_events").description("List all the events on the server")
}
