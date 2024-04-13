use std::str::FromStr;

use chrono::{Datelike, Days, NaiveDate, Utc};
use google_calendar3::api::{Event, EventDateTime};
use serenity::all::{
    CommandOptionType, Context, CreateCommand, CreateCommandOption, GuildId, ResolvedOption,
    ResolvedValue,
};
use tracing::{error, instrument};

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
            NaiveDate::from_str(&date).unwrap()
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
            date: Some(date.checked_add_days(Days::new(1)).unwrap()),
            ..Default::default()
        }),
        recurrence: Some(vec!["RRULE:FREQ=YEARLY".into()]),
        ..Default::default()
    };

    let lock = ctx.data.read().await;
    let calendar_client = lock.get::<crate::calendar::Client>().unwrap();
    calendar_client.create_event(event, guild_id).await;

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
}
