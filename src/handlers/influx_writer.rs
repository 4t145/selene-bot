use std::sync::Arc;

use chrono::{DateTime, Utc};
use influxdb::Client;
use influxdb::InfluxDbWriteable;
use qqbot_sdk::bot::Bot;
use qqbot_sdk::event::handler::EventHandler;
use qqbot_sdk::event::model::Event;
use qqbot_sdk::model::*;
#[derive(Debug, Clone)]
pub struct InfluxDbClient {
    pub client: Client,
    pub database: String,
}

impl InfluxDbClient {
    pub fn new(url: &str, database: &str) -> Self {
        let client = Client::new(url, database);
        InfluxDbClient {
            client,
            database: database.to_string(),
        }
    }

    pub async fn write_message(&self, message: &MessageBotRecieved) -> Result<(), influxdb::Error> {
        let influx_message: InfluxDbMessage = message.into();
        self.client
            .query(influx_message.into_query("message"))
            .await?;
        Ok(())
    }
}

#[derive(InfluxDbWriteable)]
pub struct InfluxDbMessage {
    pub ron: String,
    pub channel_id: ChannelId,
    pub message_id: String,
    pub author_id: u64,
    pub author_name: String,
    pub author_avatar: Option<String>,
    pub time: DateTime<Utc>,
}

impl From<&MessageBotRecieved> for InfluxDbMessage {
    fn from(val: &MessageBotRecieved) -> Self {
        InfluxDbMessage {
            ron: ron::to_string(val).unwrap(),
            channel_id: val.channel_id,
            message_id: val.id.to_string(),
            author_id: val.author.id,
            author_name: val.author.username.clone(),
            author_avatar: val.author.avatar.clone(),
            time: val.timestamp,
        }
    }
}

#[derive(Debug)]
pub struct MessageWriter {
    pub client: InfluxDbClient,
}

impl MessageWriter {
    pub fn new(client: InfluxDbClient) -> Self {
        MessageWriter { client }
    }
}
impl EventHandler for MessageWriter {
    async fn handle(&self, event: Event, _bot: &Bot) -> Result<(), qqbot_sdk::Error> {
        if let Event::MessageCreate(message) = event {
            let client = self.client.clone();
            client.write_message(&message).await.map_err(|e| {
                qqbot_sdk::Error::unexpected(format!("write message to influxdb error: {:?}", e))
            })?;
        }
        Ok(())
    }
    fn would_handle(&self, event: &Event, bot: &Bot<()>) -> bool {
        matches!(event, Event::MessageCreate(_))
    }
}
