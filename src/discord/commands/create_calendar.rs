use serenity::all::{Context, CreateCommand, GuildId, Permissions, ResolvedOption};
use tracing::{info, instrument};

use crate::calendar::get_calendar_url;

#[instrument]
pub async fn run(ctx: &Context, guild_id: GuildId, _options: &[ResolvedOption<'_>]) -> String {
    info!("Creating a calendar");
    let lock = ctx.data.read().await;
    let calendar_client = lock.get::<crate::calendar::Client>().expect("No calendar client found");
    let calendars = calendar_client.get_calendars_by_guild_id(&guild_id).await;
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
    calendar_client.create_calendar(&guild_id.to_string()).await;
    "Created the calendar! `/list_events` to get the url".into()
}

pub fn register() -> CreateCommand {
    CreateCommand::new("create_calendar")
        .description("Create a new event calendar for the server, removing the old")
        .default_member_permissions(Permissions::ADMINISTRATOR)
}
