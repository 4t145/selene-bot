use std::sync::Arc;

use chrono::{DateTime, Utc};
use influxdb::InfluxDbWriteable;
use influxdb::{Client};
use qqbot_sdk::bot::{Bot, BotError, Handler};
use qqbot_sdk::model::*;
use qqbot_sdk::websocket::ClientEvent;
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
impl Handler for MessageWriter {
    fn handle(&self, event: ClientEvent, _ctx: Arc<Bot>) -> Result<(), BotError> {
        if let ClientEvent::MessageCreate(message) = event {
            let client = self.client.clone();
            tokio::spawn(async move {
                client.write_message(&message).await.unwrap();
            });
        }
        Ok(())
    }
}
