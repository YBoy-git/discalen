use std::collections::HashMap;

use serenity::{
    all::{ChannelId, GuildId},
    prelude::TypeMapKey,
};

#[derive(Default)]
pub struct Data {
    pub event_channels: HashMap<GuildId, ChannelId>,
}

impl TypeMapKey for Data {
    type Value = Self;
}

pub struct Client {
    pub serenity_client: serenity::Client,
}

impl Client {
    pub async fn new(client: serenity::Client) -> Self {
        {
            let mut data = client.data.write().await;
            data.insert::<Data>(Data::default());
        }
        Self {
            serenity_client: client,
        }
    }
}
