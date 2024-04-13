use std::str::FromStr;

use crate::{calendar::Client as CalendarClient, Error};
use chrono::{Datelike, Days, NaiveDate, Utc};
use google_calendar3::api::{Event, EventDateTime};
use serenity::all::{
    CommandOptionType, Context, CreateCommand, CreateCommandOption, GuildId, Permissions,
    ResolvedOption, ResolvedValue,
};
use tracing::{instrument, warn};

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

    let date = match options.get(1) {
        Some(ResolvedOption {
            value: ResolvedValue::String(date),
            ..
        }) => {
            let year = Utc::now().year();
            let date = format!("{year}-{date}");
            NaiveDate::from_str(&date)?
        }
        _ => Utc::now().date_naive(),
    };

    let event = Event {
        summary: Some(label.to_string()),
        start: Some(EventDateTime {
            date: Some(date),
            ..Default::default()
        }),
        end: Some(EventDateTime {
            date: Some(
                date.checked_add_days(Days::new(1))
                    .expect("Out of range days"),
            ),
            ..Default::default()
        }),
        recurrence: Some(vec!["RRULE:FREQ=YEARLY".into()]),
        ..Default::default()
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
    calendar_client.create_event(event, &calendar_id).await?;

    Ok(format!(
        "The event \"{label}\" was created successfully! Date: {date}"
    ))
}

pub fn register() -> CreateCommand {
    CreateCommand::new("create_event")
        .description("Create an event with a label on a specific date")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "label", "The label of the event")
                .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "date",
                "The date of the event using the MM-DD format. Default is today",
            )
            .required(false),
        )
        .default_member_permissions(Permissions::ADMINISTRATOR)
}
