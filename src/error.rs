use serenity::all::GuildId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("The calendar summary {0} is not a discord server id")]
    CalendarSummaryNotDiscordServerId(String),

    #[error("The server {0} has no event channel")]
    DiscordSeverHasNoEventChannel(GuildId),

    #[error("No pool in data")]
    NoPool,

    #[error("No calendar client in data")]
    NoCalendarClient,

    #[error(transparent)]
    DbError(#[from] sqlx::Error),

    #[error(transparent)]
    GoogleError(#[from] google_calendar3::Error),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    ConfigError(#[from] config::ConfigError),

    #[error(transparent)]
    SerenityError(#[from] serenity::Error),
}
