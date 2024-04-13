use crate::calendar::Client as CalendarClient;
use serenity::all::{
    CommandOptionType, Context, CreateCommand, CreateCommandOption, GuildId, Permissions,
    ResolvedOption, ResolvedValue,
};
use tracing::{error, info};

pub async fn run(ctx: &Context, guild_id: &GuildId, options: &[ResolvedOption<'_>]) -> String {
    let Some(ResolvedOption {
        value: ResolvedValue::String(label),
        ..
    }) = options.first()
    else {
        error!("Missing the label parameter");
        return "The *label* parameter is required".to_string();
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
        error!("Couldn't find a calendar for the guild");
        return "No calendar for the server, create a new one! `/create_calendar`".into();
    };
    let calendar_id = calendar.id.unwrap_or_else(|| unreachable!());

    let event_ids = match calendar_client
        .get_event_id_by_label(label, &calendar_id)
        .await
    {
        Ok(ids) => ids,
        Err(why) => {
            error!(?why, "Failed to get event id by label");
            return format!("An error occurred: {why}");
        }
    };
    info!(?event_ids, "Deleting these events");

    let mut handles = Vec::with_capacity(event_ids.len());
    for id in &event_ids {
        let handle = calendar_client.delete_event(id, &calendar_id);
        handles.push(handle);
    }
    futures::future::join_all(handles).await;

    format!("Deleted {} events from the calendar!", event_ids.len())
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
