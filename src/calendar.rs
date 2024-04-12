use google_calendar3::{
    api::{Calendar, Event},
    hyper, hyper_rustls, CalendarHub,
};
use serenity::{all::GuildId, prelude::TypeMapKey};
use tracing::{error, info};
use yup_oauth2::{
    hyper::Client as CalendarClient, read_service_account_key, ServiceAccountAuthenticator,
};

pub type MyCalendarHub =
    CalendarHub<hyper_rustls::HttpsConnector<hyper::client::connect::HttpConnector>>;

pub async fn authenticate_calendar_hub() -> MyCalendarHub {
    let sa_key = read_service_account_key(format!(
        "{}/secrets/discalen-sa.json",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    ))
    .await
    .expect("Failed to read service account key");
    let auth = ServiceAccountAuthenticator::builder(sa_key)
        .build()
        .await
        .expect("Failed to create authenticator");

    let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_or_http()
        .enable_http1()
        .build();
    let client = CalendarClient::builder().build(https_connector);

    CalendarHub::new(client, auth)
}

#[derive(Clone)]
pub struct Client {
    pub calendar_hub: MyCalendarHub,
}

impl TypeMapKey for Client {
    type Value = Self;
}

impl Client {
    pub async fn default() -> Self {
        Self {
            calendar_hub: authenticate_calendar_hub().await,
        }
    }
}

impl Client {
    pub async fn create_calendar(&self, name: &str) {
        let _ = match self
            .calendar_hub
            .calendars()
            .insert(Calendar {
                summary: Some(name.to_string()),
                ..Default::default()
            })
            .doit()
            .await
        {
            Ok(res) => res.1,
            Err(err) => {
                error!("Error creating a calendar: {err}");
                return;
            }
        };
    }

    pub async fn list_calendars(&self) -> Vec<String> {
        self.calendar_hub
            .calendar_list()
            .list()
            .doit()
            .await
            .unwrap()
            .1
            .items
            .unwrap()
            .into_iter()
            .map(|calendar| calendar.id.unwrap().to_string())
            .collect()
    }

    pub async fn list_events(&self, calendar_id: &str) -> Vec<Event> {
        self.calendar_hub
            .events()
            .list(calendar_id)
            .doit()
            .await
            .unwrap()
            .1
            .items
            .unwrap()
    }

    pub async fn get_calendar_id_by_guild_id(&self, guild_id: GuildId) -> Option<String> {
        let mut response = self
            .calendar_hub
            .calendar_list()
            .list()
            .doit()
            .await
            .unwrap()
            .1
            .items
            .unwrap();
        response.retain(|calendar| calendar.summary == Some(guild_id.to_string()));
        assert!(response.len() <= 1);
        response.pop()?.id
    }

    pub async fn get_event_id_by_label(&self, label: &str, guild_id: GuildId) -> Vec<String> {
        let calendar_id = self.get_calendar_id_by_guild_id(guild_id).await.unwrap();
        let response = self
            .calendar_hub
            .events()
            .list(&calendar_id)
            .doit()
            .await
            .unwrap()
            .1;
        info!(?response, "Got the events to delete");
        response
            .items
            .unwrap()
            .into_iter()
            .filter_map(|event| {
                info!(event = event.summary, "Scanning an event...");
                if event.summary == Some(label.to_string()) {
                    info!("Pushing...");
                    Some(event.id.unwrap())
                } else {
                    None
                }
            })
            .collect()
    }
}
