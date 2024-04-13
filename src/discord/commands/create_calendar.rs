use crate::calendar::Client as CalendarClient;
use serenity::all::{Context, CreateCommand, GuildId, Permissions, ResolvedOption};
use tracing::{error, info, instrument};

use crate::calendar::get_calendar_url;

#[instrument]
pub async fn run(ctx: &Context, guild_id: GuildId, _options: &[ResolvedOption<'_>]) -> String {
    info!("Creating a calendar");
    let lock = ctx.data.read().await;
    let calendar_client = match lock
        .get::<CalendarClient>() {
            Some(client) => client,
            None => {
                error!("No calendar client found");
                return "An error occurred: no calendar client found".into();
            }
        };
    let calendars = match calendar_client.get_calendars_by_guild_id(&guild_id).await {
        Ok(calendars) => calendars,
        Err(why) => {
            error!(?why, "Failed to get calendars by id");
            return format!("An error occurred: {why}");
        }
    };
    if !calendars.is_empty() {
        return format!(
            "A calendar already exists: {}",
            calendars
                .into_iter()
                .map(|calendar| {
                    let calendar_id = calendar.id.unwrap_or_else(|| unreachable!());
                    get_calendar_url(&calendar_id)
                })
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if let Err(why) = calendar_client.create_calendar(&guild_id.to_string()).await {
        error!(?why, "Failed to create calendar");
        return format!("An error occurred: {why}");
    };
    "Created the calendar! `/list_events` to get the url".into()
}

pub fn register() -> CreateCommand {
    CreateCommand::new("create_calendar")
        .description("Create a new event calendar for the server, removing the old")
        .default_member_permissions(Permissions::ADMINISTRATOR)
}
