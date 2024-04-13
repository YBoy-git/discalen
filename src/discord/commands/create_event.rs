use std::str::FromStr;

use crate::calendar::Client as CalendarClient;
use chrono::{Datelike, Days, NaiveDate, Utc};
use google_calendar3::api::{Event, EventDateTime};
use serenity::all::{
    CommandOptionType, Context, CreateCommand, CreateCommandOption, GuildId, Permissions,
    ResolvedOption, ResolvedValue,
};
use tracing::{error, instrument, warn};

#[instrument]
pub async fn run(ctx: &Context, guild_id: &GuildId, options: &[ResolvedOption<'_>]) -> String {
    let Some(ResolvedOption {
        value: ResolvedValue::String(label),
        ..
    }) = options.first()
    else {
        error!("Missing the label parameter");
        return "The *label* parameter is required".to_string();
    };

    let date = match options.get(1) {
        Some(ResolvedOption {
            value: ResolvedValue::String(date),
            ..
        }) => {
            let year = Utc::now().year();
            let date = format!("{year}-{date}");
            match NaiveDate::from_str(&date) {
                Ok(date) => date,
                Err(err) => {
                    error!(date, "Wrong format for the date passed");
                    return format!("Failed to parse date: {err}. Date format is MM-DD");
                }
            }
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
    let calendar_client = match lock.get::<CalendarClient>() {
        Some(client) => client,
        None => {
            error!("No calendar client");
            return "An error occurred: no calendar client".into();
        }
    };

    let mut calendars = match calendar_client.get_calendars_by_guild_id(guild_id).await {
        Ok(calendars) => calendars,
        Err(why) => {
            error!(?why, "Failed to get calendars by guild id");
            return format!("An error occurred: {why}");
        }
    };
    let Some(calendar) = calendars.pop() else {
        warn!("Couldn't find a calendar for the guild");
        return "No calendar for the server, create a new one! `/create_calendar`".into();
    };
    let calendar_id = calendar.id.unwrap_or_else(|| unreachable!());
    if let Err(why) = calendar_client.create_event(event, &calendar_id).await {
        error!(?why, "Failed to create event");
        return format!("An error occurred: {why}");
    };

    format!("The event \"{label}\" was created successfully! Date: {date}")
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
