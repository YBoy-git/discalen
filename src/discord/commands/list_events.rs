use serenity::all::{Context, CreateCommand, GuildId, ResolvedOption};
use tracing::{info, instrument};

#[instrument]
pub async fn run(ctx: &Context, guild_id: &GuildId, _options: &[ResolvedOption<'_>]) -> String {
    info!("Fetching an event list for a guild");
    let lock = ctx.data.read().await;
    let calendar_client = lock.get::<crate::calendar::Client>().unwrap();

    let events = calendar_client
        .list_events(
            &calendar_client
                .get_calendar_id_by_guild_id(guild_id)
                .await
                .unwrap(),
        )
        .await;

    info!(?events, "Returned event list");

    format!(
        "Events:\n{}",
        events
            .into_iter()
            .map(|event| {
                format!(
                    "{}: {}",
                    event.summary.unwrap(),
                    event.start.unwrap().date.unwrap()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    )
}

pub fn register() -> CreateCommand {
    CreateCommand::new("list_events").description("List all the events on the server")
}