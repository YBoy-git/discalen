use serenity::all::{ChannelId, GuildId};
use sqlx::{query, PgPool};

pub struct Client {
    pub serenity_client: serenity::Client,
}

impl Client {
    pub async fn new(client: serenity::Client) -> Self {
        Self {
            serenity_client: client,
        }
    }
}

pub async fn set_event_channel(pool: &PgPool, guild_id: &GuildId, channel_id: &ChannelId) {
    query!(
        "
        DELETE FROM event_channels WHERE guild_id = $1
        ",
        guild_id.get().to_string()
    )
    .execute(pool)
    .await
    .unwrap();
    query!(
        "
        INSERT INTO event_channels(guild_id, channel_id)
        VALUES($1, $2)
        ON CONFLICT DO NOTHING
        ",
        guild_id.get().to_string(),
        channel_id.get().to_string(),
    )
    .execute(pool)
    .await
    .unwrap();
}

pub async fn get_event_channel_id(pool: &PgPool, guild_id: &GuildId) -> Option<ChannelId> {
    query!(
        "
        SELECT channel_id FROM event_channels
        WHERE guild_id = $1
        ",
        guild_id.get().to_string()
    )
    .fetch_optional(pool)
    .await
    .unwrap()
    .and_then(|record| Some(ChannelId::from(record.channel_id.parse::<u64>().ok()?)))
}
