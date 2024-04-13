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
    let calendar_client = lock
        .get::<CalendarClient>()
        .expect("No calendar client found");
    let Some(calendar) = calendar_client
        .get_calendars_by_guild_id(guild_id)
        .await
        .pop()
    else {
        error!("Couldn't find a calendar for the guild");
        return "No calendar for the server, create a new one! `/create_calendar`".into();
    };
    let calendar_id = calendar.id.unwrap_or_else(|| unreachable!());

    let event_ids = calendar_client
        .get_event_id_by_label(label, &calendar_id)
        .await;
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
