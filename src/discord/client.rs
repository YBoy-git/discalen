use std::collections::{HashMap, VecDeque};

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

#[derive(Default)]
pub struct Shared {
    pub list_requests: VecDeque<GuildId>,
}

impl TypeMapKey for Shared {
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
            data.insert::<Shared>(Shared::default());
        }
        Self {
            serenity_client: client,
        }
    }
}
