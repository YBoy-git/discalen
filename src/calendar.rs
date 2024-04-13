use google_calendar3::{
    api::{AclRule, AclRuleScope, Calendar, CalendarListEntry, Event},
    hyper, hyper_rustls, CalendarHub,
};
use serenity::{all::GuildId, prelude::TypeMapKey};
use tracing::{error, instrument, warn};
use yup_oauth2::{
    hyper::Client as CalendarClient, parse_service_account_key, ServiceAccountAuthenticator,
};

use crate::Error;

pub type MyCalendarHub =
    CalendarHub<hyper_rustls::HttpsConnector<hyper::client::connect::HttpConnector>>;

pub async fn authenticate_calendar_hub(
    key: impl AsRef<[u8]>,
) -> Result<MyCalendarHub, Error> {
    let sa_key = parse_service_account_key(key)?;
    let auth = ServiceAccountAuthenticator::builder(sa_key).build().await?;

    let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_or_http()
        .enable_http1()
        .build();
    let client = CalendarClient::builder().build(https_connector);

    Ok(CalendarHub::new(client, auth))
}

#[derive(Clone)]
pub struct Client {
    pub calendar_hub: MyCalendarHub,
}

impl TypeMapKey for Client {
    type Value = Self;
}

impl Client {
    pub async fn with_sa_key(key: impl AsRef<[u8]>) -> Result<Self, Error> {
        Ok(Self {
            calendar_hub: authenticate_calendar_hub(key.as_ref()).await?,
        })
    }
}

impl Client {
    #[instrument(skip(self))]
    pub async fn create_calendar(&self, name: &str) -> Result<Calendar, Error> {
        let calendar = Calendar {
            summary: Some(name.to_string()),
            ..Default::default()
        };
        let calendar = match self.calendar_hub.calendars().insert(calendar).doit().await {
            Ok(res) => res.1,
            Err(why) => {
                error!("Error creating a calendar: {why}");
                return Err(Error::GoogleError(why));
            }
        };

        let rule = AclRule {
            role: Some("reader".into()),
            scope: Some(AclRuleScope {
                type_: Some("default".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        if let Err(why) = self
            .calendar_hub
            .acl()
            .insert(rule, calendar.id.as_ref().unwrap_or_else(|| unreachable!()))
            .doit()
            .await
        {
            error!(?why, "Failed to set acl for the calendar");
            return Err(Error::GoogleError(why));
        };
        Ok(calendar)
    }

    #[instrument(skip(self))]
    pub async fn delete_calendar(&self, calendar_id: &str) -> Result<(), Error> {
        if let Err(why) = self
            .calendar_hub
            .calendars()
            .delete(calendar_id)
            .doit()
            .await
        {
            error!(?why, "Failed to delete the calendar");
            return Err(Error::GoogleError(why));
        };
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn create_event(&self, event: Event, calendar_id: &str) -> Result<Event, Error> {
        match self
            .calendar_hub
            .events()
            .insert(event, calendar_id)
            .doit()
            .await
        {
            Ok(event) => Ok(event.1),
            Err(why) => {
                error!(?why, "Failed to create an event");
                Err(Error::GoogleError(why))
            }
        }
    }

    #[instrument(skip(self))]
    pub async fn delete_event(&self, id: &str, calendar_id: &str) -> Result<(), Error> {
        if let Err(why) = self
            .calendar_hub
            .events()
            .delete(calendar_id, id)
            .doit()
            .await
        {
            error!(?why, "Failed to delete an event");
            return Err(Error::GoogleError(why));
        };
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn list_calendars(&self) -> Result<Vec<CalendarListEntry>, Error> {
        match self.calendar_hub.calendar_list().list().doit().await {
            Ok(response) => Ok(response.1.items.unwrap_or_else(|| unreachable!())),
            Err(why) => {
                error!(?why, "Failed to list calendars");
                Err(Error::GoogleError(why))
            }
        }
    }

    #[instrument(skip(self))]
    pub async fn list_events(&self, calendar_id: &str) -> Result<Vec<Event>, Error> {
        match self.calendar_hub.events().list(calendar_id).doit().await {
            Ok(response) => Ok(response.1.items.unwrap_or_else(|| unreachable!())),
            Err(why) => {
                error!(?why, "Failed to list events");
                Err(Error::GoogleError(why))
            }
        }
    }

    #[instrument(skip(self))]
    pub async fn get_calendars_by_guild_id(
        &self,
        guild_id: &GuildId,
    ) -> Result<Vec<CalendarListEntry>, Error> {
        let mut response = self.list_calendars().await?;
        response.retain(|calendar| calendar.summary == Some(guild_id.to_string()));
        if response.len() > 1 {
            warn!("More than one calendar associated with the server, using the newest one");
        };
        Ok(response)
    }

    #[instrument(skip(self))]
    pub async fn get_event_id_by_label(
        &self,
        label: &str,
        calendar_id: &str,
    ) -> Result<Vec<String>, Error> {
        let response = self.list_events(calendar_id).await?;
        let value = response
            .into_iter()
            .filter_map(|event| {
                if event.summary == Some(label.to_string()) {
                    Some(event.id.unwrap_or_else(|| unreachable!()))
                } else {
                    None
                }
            })
            .collect();
        Ok(value)
    }
}

pub fn get_calendar_url(calendar_id: &str) -> String {
    format!("https://calendar.google.com/calendar/u/0?cid={calendar_id}")
}
