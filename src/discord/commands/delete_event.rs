use serenity::all::{
    CommandOptionType, Context, CreateCommand, CreateCommandOption, GuildId, ResolvedOption,
    ResolvedValue,
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
    let calendar_client = lock.get::<crate::calendar::Client>().unwrap();
    let event_ids = calendar_client.get_event_id_by_label(label, guild_id).await;
    info!(?event_ids, "Deleting these events");
    for id in &event_ids {
        calendar_client.delete_event(id, guild_id).await;
    }

    format!("Deleted {} events from the calendar!", event_ids.len())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("delete_event")
        .description("Delete events with the specified label")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "label", "The label of the event")
                .required(true),
        )
}
